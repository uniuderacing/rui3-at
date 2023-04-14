use atat::atat_derive::AtatResp;
use crate::commands::p2p::WorkingMode;

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