//! Example that runs on Linux using a serial-USB-adapter.
use std::{
    env, io,
    net::{SocketAddr as StdSocketAddr, ToSocketAddrs},
    thread,
    time::Duration,
};

use atat::bbqueue::BBBuffer;
use embedded_nal::{SocketAddr as NalSocketAddr, TcpClientStack};
use esp_at_nal::{
    urc::URCMessages,
    wifi::{Adapter, WifiAdapter},
};
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

// Chunk size in bytes when sending data. Higher value results in better
// performance, but introduces also higher stack memory footprint. Max value: 8192.
const TX_SIZE: usize = 1024;
// Chunk size in bytes when receiving data. Value should be matched to buffer
// size of receive() calls.
const RX_SIZE: usize = 2048;

// Constants derived from TX_SIZE and RX_SIZE
const ESP_TX_SIZE: usize = TX_SIZE;
const ESP_RX_SIZE: usize = RX_SIZE;
const ATAT_RX_SIZE: usize = RX_SIZE;
const URC_RX_SIZE: usize = RX_SIZE;
const RES_CAPACITY: usize = RX_SIZE;
const URC_CAPACITY: usize = RX_SIZE * 3;

// Timer frequency in Hz
const TIMER_HZ: u32 = 1000;

fn main() {
    // Parse args
    let args: Vec<String> = env::args().collect();
    if args.len() != 5 {
        println!("Usage: {} <path-to-serial> <baudrate> <ssid> <psk>", args[0]);
        println!("Example: {} /dev/ttyUSB0 115200 mywifi hellopasswd123", args[0]);
        println!("\nNote: To run the example with debug logging, run it like this:");
        println!("\n  RUST_LOG=trace cargo run --example linux --features \"atat/log\" -- /dev/ttyUSB0 115200 mywifi hellopasswd123");
        std::process::exit(1);
    }
    let dev = &args[1];
    let baud_rate: u32 = args[2].parse().unwrap();
    let ssid = &args[3];
    let psk = &args[4];

    println!("Starting (dev={}, baud={:?})...", dev, baud_rate);

    // Open serial port
    let serial_tx = serialport::new(dev, baud_rate)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(500))
        .open()
        .expect("Could not open serial port");
    let mut serial_rx = serial_tx.try_clone().expect("Could not clone serial port");

    // Atat queues
    static mut RES_QUEUE: BBBuffer<RES_CAPACITY> = BBBuffer::new();
    static mut URC_QUEUE: BBBuffer<URC_CAPACITY> = BBBuffer::new();
    let queues = atat::Queues {
        res_queue: unsafe { RES_QUEUE.try_split_framed().unwrap() },
        urc_queue: unsafe { URC_QUEUE.try_split_framed().unwrap() },
    };

    // Two timer instances
    let atat_timer = timer::SysTimer::new();
    let esp_timer = timer::SysTimer::new();

    // Atat client
    let config = atat::Config::new(atat::Mode::Timeout);
    let digester = atat::AtDigester::<URCMessages>::new();
    let (client, mut ingress) =
        atat::ClientBuilder::<_, _, _, TIMER_HZ, ATAT_RX_SIZE, RES_CAPACITY, URC_CAPACITY>::new(
            serial_tx, atat_timer, digester, config,
        )
        .build(queues);

    // Flush serial RX buffer, to ensure that there isn't any remaining left
    // form previous sessions.
    flush_serial(&mut serial_rx);

    // Launch reading thread, to pass incoming data from serial to the atat ingress
    thread::Builder::new()
        .name("serial_read".to_string())
        .spawn(move || loop {
            let mut buffer = [0; 32];
            match serial_rx.read(&mut buffer[..]) {
                Ok(0) => {}
                Ok(bytes_read) => {
                    ingress.write(&buffer[0..bytes_read]);
                    ingress.digest();
                    ingress.digest();
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut | io::ErrorKind::Interrupted => {
                        // Ignore
                    }
                    _ => {
                        println!("Serial reading thread error while reading: {}", e);
                    }
                },
            }
        })
        .unwrap();


}

/// Flush the serial port receive buffer.
fn flush_serial(serial_rx: &mut Box<dyn SerialPort>) {
    let mut buf = [0; 32];
    loop {
        match serial_rx.read(&mut buf[..]) {
            Ok(0) => break,
            Err(e) if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut => break,
            Ok(_) => continue,
            Err(e) => panic!("Error while flushing serial: {}", e),
        }
    }
}

mod timer {
    use std::{convert::TryInto, time::Instant as StdInstant};

    use atat::clock::Clock;
    use fugit::Instant;

    /// A timer with millisecond precision.
    pub struct SysTimer {
        start: StdInstant,
        duration_ms: u32,
        started: bool,
    }

    impl SysTimer {
        pub fn new() -> SysTimer {
            SysTimer {
                start: StdInstant::now(),
                duration_ms: 0,
                started: false,
            }
        }
    }

    impl Clock<1000> for SysTimer {
        type Error = &'static str;

        /// Return current time `Instant`
        fn now(&mut self) -> fugit::TimerInstantU32<1000> {
            let milliseconds = (StdInstant::now() - self.start).as_millis();
            let ticks: u32 = milliseconds.try_into().expect("u32 timer overflow");
            Instant::<u32, 1, 1000>::from_ticks(ticks)
        }

        /// Start timer with a `duration`
        fn start(&mut self, duration: fugit::TimerDurationU32<1000>) -> Result<(), Self::Error> {
            // (Re)set start and duration
            self.start = StdInstant::now();
            self.duration_ms = duration.ticks();

            // Set started flag
            self.started = true;

            Ok(())
        }

        /// Tries to stop this timer.
        ///
        /// An error will be returned if the timer has already been canceled or was never started.
        /// An error is also returned if the timer is not `Periodic` and has already expired.
        fn cancel(&mut self) -> Result<(), Self::Error> {
            if !self.started {
                Err("cannot cancel stopped timer")
            } else {
                self.started = false;
                Ok(())
            }
        }

        /// Wait until timer `duration` has expired.
        /// Must return `nb::Error::WouldBlock` if timer `duration` is not yet over.
        /// Must return `OK(())` as soon as timer `duration` has expired.
        fn wait(&mut self) -> nb::Result<(), Self::Error> {
            let now = StdInstant::now();
            if (now - self.start).as_millis() > self.duration_ms.into() {
                Ok(())
            } else {
                Err(nb::Error::WouldBlock)
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn test_delay() {
            let mut timer = SysTimer::new();

            // Wait 500 ms
            let before = StdInstant::now();
            timer.start(fugit::Duration::<u32, 1, 1000>::from_ticks(500)).unwrap();
            nb::block!(timer.wait()).unwrap();
            let after = StdInstant::now();

            let duration_ms = (after - before).as_millis();
            assert!(duration_ms >= 500);
            assert!(duration_ms < 1000);
        }
    }
}


