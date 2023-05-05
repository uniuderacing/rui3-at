use std::io::BufRead;
use atat::atat_derive::{AtatCmd, AtatResp};

// static mut INGRESS: Option<atat::IngressManager> = None;
// static mut RX: Option<Rx<USART2>> = None;


#[derive(Clone, AtatResp)]
pub struct NoResponse;

#[derive(Clone, AtatCmd)]
#[at_cmd("", NoResponse, timeout_ms = 1000)]
pub struct AT;




fn main() {
    
    let mut serial_ports = serialport::available_ports().unwrap();
    let serial_port = serialport::new(serial_ports.pop().unwrap().port_name, 115_200)
        .timeout(std::time::Duration::from_millis(500))
        .open_native()
        .unwrap();

    let (mut tx, mut rx) = serial_port.split();
    
    let timer = timer::SysTimer::new();
    let digester = atat::AtDigester::<URCMessages<>>::new();
    let config = atat::Config::new(atat::Mode::Timeout);

    let mut at_client = atat::ClientBuilder::new(tx, timer, digester, config);

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
