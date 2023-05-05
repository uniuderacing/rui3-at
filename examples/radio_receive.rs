use std::io;
use std::io::BufRead;
use std::time::Duration;
use atat::atat_derive::{AtatCmd, AtatResp};
use atat::AtDigester;
use atat::bbqueue::BBBuffer;
use serialport::*;

// static mut INGRESS: Option<atat::IngressManager> = None;
// static mut RX: Option<Rx<USART2>> = None;
//static mut INGRESS: Option<atat::IngressManager> = None;
//static mut RX: Option<Rx<USART2>> = None;

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
    
    let mut serial_ports = serialport::available_ports().unwrap();
    let serial_tx = serialport::new(serial_ports.pop().unwrap().port_name, 115_200)
        .data_bits(DataBits::Eight)
        .flow_control(FlowControl::None)
        .parity(Parity::None)
        .stop_bits(StopBits::One)
        .timeout(Duration::from_millis(500))
        .open()
        .expect("Could not open serial port");

    let mut serial_rx = serial_tx.try_clone().expect("Could not clone serial port");

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
    let digester: AtDigester<> = atat::AtDigester::new();
    let (client, mut ingress) =
        atat::ClientBuilder::<_, _, _, TIMER_HZ, ATAT_RX_SIZE, RES_CAPACITY, URC_CAPACITY>::new(
            serial_tx, atat_timer, digester, config,
        )
            .build(queues);

    // Flush serial RX buffer, to ensure that there isn't any remaining left
    // form previous sessions.
    flush_serial(&mut serial_rx);

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
}


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



