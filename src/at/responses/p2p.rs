use crate::at::commands::p2p::{Bandwidth, CodeRate, WorkingMode, Encrypted};
use atat::atat_derive::AtatResp;

#[derive(Clone, AtatResp)]
pub struct NetworkWorkingModeResponse {
    #[at_arg(position = 0)]
    pub mode: WorkingMode,
}

#[derive(Clone, AtatResp)]
pub struct P2PFrequencyResponse {
    #[at_arg(position = 0)]
    pub frequency: u32,
}

#[derive(Clone, AtatResp)]
pub struct P2PSpreadingFactorResponse {
    #[at_arg(position = 0)]
    pub spreading_factor: u8,
}

#[derive(Clone, AtatResp)]
pub struct P2PBandwidth {
    #[at_arg(position = 0)]
    pub bandwidth: Bandwidth,
}

#[derive(Clone, AtatResp)]
pub struct P2PCodeRate {
    #[at_arg(position = 0)]
    pub code_rate: CodeRate,
}

#[derive(Clone, AtatResp)]
pub struct P2PPreambleLength {
    #[at_arg(position = 0)]
    pub preamble_length: u16,
}

#[derive(Clone, AtatResp)]
pub struct P2PTxPower {
    #[at_arg(position = 0)]
    pub tx_power: u8,
}

#[derive(Clone, AtatResp)]
pub struct P2PEncryptionMode {
    #[at_arg(position = 0)]
    pub encryption: Encrypted,
}

#[derive(Clone, AtatResp)]
pub struct P2PEncryptionKey {
    #[at_arg(position = 0)]
    pub encryption_key: atat::heapless::String<16>,
}

#[derive(Clone, AtatResp)]
pub struct P2Pparameters {
    #[at_arg(position = 0)]
    pub frequency: u32,
    pub spreading_factor: u8,
    pub bandwidth: Bandwidth,
    pub coderate: CodeRate,
    pub preamblelength: u16,
    pub txpower: u8,
}

#[derive(Clone, AtatResp)]
pub struct P2PIQInversion {
    #[at_arg(position = 0)]
    pub iq_inversion: bool,
}

#[derive(Clone, AtatResp)]
pub struct P2PSyncWord {
    #[at_arg(position = 0)]
    pub sync_word: atat::heapless::String<4>,
}

// #[derive(Clone, AtatResp)]
// pub struct P2PFrequency {
//     #[at_arg(position = 0)]
//     pub frequency: u32,
// }

#[derive(Clone, AtatResp)]
pub struct P2PSymbolTimeout {
    #[at_arg(position = 0)]
    pub symbol_timeout: u8,
}
