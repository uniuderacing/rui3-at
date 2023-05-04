use atat::atat_derive::{AtatCmd, AtatResp};

// static mut INGRESS: Option<atat::IngressManager> = None;
//static mut RX: Option<Rx<USART2>> = None;


#[derive(Clone, AtatResp)]
pub struct NoResponse;

#[derive(Clone, AtatCmd)]
#[at_cmd("", NoResponse, timeout_ms = 1000)]
pub struct AT;


fn main() {
    let mut serial_ports = serialport::available_ports().unwrap();
    let serial_port = serialport::new(serial_ports.pop().unwrap().port_name, 115_200)
        .timeout(std::time::Duration::from_millis(10))
        .open_native()
        .unwrap();

}


