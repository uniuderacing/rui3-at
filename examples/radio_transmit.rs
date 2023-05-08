//! Example that runs on Linux using a serial-USB-adapter.
use std::{io, thread, time::Duration};

use atat::bbqueue::BBBuffer;
use rui3_at::Configuration;
use serialport::{DataBits, FlowControl, Parity, SerialPort, StopBits};

// Chunk size in bytes when sending data. Higher value results in better
// performance, but introduces also higher stack memory footprint. Max value: 8192.
// const TX_SIZE: usize = 1024;
// Chunk size in bytes when receiving data. Value should be matched to buffer
// size of receive() calls.
const RX_SIZE: usize = 2048;

// Constants derived from TX_SIZE and RX_SIZE
const ATAT_RX_SIZE: usize = RX_SIZE;
// const URC_RX_SIZE: usize = RX_SIZE;
const RES_CAPACITY: usize = RX_SIZE;
const URC_CAPACITY: usize = RX_SIZE * 3;

// Timer frequency in Hz
const TIMER_HZ: u32 = 1000;

fn main() {
    // TODO: Add support for command line arguments
    // Print available ports
    let ports = serialport::available_ports().expect("No serial ports found!");
    for p in ports {
        println!("{}", p.port_name);
    }

    // Open serial port
    let mut serial_tx = serialport::new("/dev/cu.usbserial-0001", 115200)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(100))
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

    // Atat client
    let config = atat::Config::new(atat::Mode::Blocking);
    let digester = atat::AtDigester::<rui3_at::at::urc::URCMessages>::new();
    let (client, mut ingress) =
        atat::ClientBuilder::<_, _, _, TIMER_HZ, ATAT_RX_SIZE, RES_CAPACITY, URC_CAPACITY>::new(
            serial_tx, atat_timer, digester, config,
        )
        .build(queues);
    println!("Atat client created");

    // Flush serial RX buffer, to ensure that there isn't any remaining left
    // form previous sessions.

    flush_serial(&mut serial_rx);

    // TODO: Add thread for serial reading
    // Launch reading thread, to pass incoming data from serial to the atat ingress
    thread::Builder::new()
        .name("serial_read".to_string())
        .spawn(move || loop {
            let mut buffer = [0; 32];
            match serial_rx.read(&mut buffer[..]) {
                Ok(0) => {}
                Ok(bytes_read) => {
                    let swapped_buffer = String::from_utf8(buffer[0..bytes_read].to_vec()).unwrap().replace("\n\r", "\r\n");
                    ingress.write(&swapped_buffer.as_bytes()[0..bytes_read]);
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

            std::thread::sleep(Duration::from_millis(1000));
        })
        .unwrap();

    // Radio data transmission thread
    thread::spawn(move || {
        // Create Rui3Radio instance
        let mut radio = rui3_at::Rui3Radio::new(client);
        println!("Radio created");

        // Configure radio
        let configuration = radio.configure(Configuration::default());
        match configuration {
            Ok(_) => println!("Configuration successful"),
            Err(e) => println!("Configuration failed: {:?}", e),
        }

        // Send data
        loop {
            let data_sent = radio.send(&[88]);
            match data_sent {
                Ok(_) => println!("Data sent"),
                Err(e) => println!("Data not sent: {:?}", e),
            }
            thread::sleep(Duration::from_millis(1000));
        }
    });

    loop {
        thread::sleep(Duration::from_millis(200));
    }
}

/// Flush the serial port receive buffer.
fn flush_serial(serial_rx: &mut Box<dyn SerialPort>) {
    let mut buf = [0; 32];
    loop {
        match serial_rx.read(&mut buf[..]) {
            Ok(0) => break,
            Err(e)
                if e.kind() == io::ErrorKind::WouldBlock || e.kind() == io::ErrorKind::TimedOut =>
            {
                break
            }
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
            timer
                .start(fugit::Duration::<u32, 1, 1000>::from_ticks(500))
                .unwrap();
            nb::block!(timer.wait()).unwrap();
            let after = StdInstant::now();

            let duration_ms = (after - before).as_millis();
            assert!(duration_ms >= 500);
            assert!(duration_ms < 1000);
        }
    }
}
