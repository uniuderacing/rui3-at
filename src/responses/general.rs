use atat::atat_derive::AtatResp;

#[derive(Clone, AtatResp)]
pub struct SerialNumberResponse {
    #[at_arg(position = 0)]
    pub serial_number: atat::heapless::String<18>,
}

#[derive(Clone, AtatResp)]
pub struct FirmwareVersionResponse {
    #[at_arg(position = 0)]
    pub firmware_version: atat::heapless::String<128>,
}

#[derive(Clone, AtatResp)]
pub struct HardwareVersionResponse {
    #[at_arg(position = 0)]
    pub hardware_version: atat::heapless::String<32>,
}

#[derive(Clone, AtatResp)]
pub struct AliasResponse {
    #[at_arg(position = 0)]
    pub alias: atat::heapless::String<16>,
}
