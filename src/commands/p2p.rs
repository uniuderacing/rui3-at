use crate::responses::{NoResponse, p2p::{P2PFrequencyResponse, P2PSpreadingFactorResponse}};
use atat::{atat_derive::{AtatCmd, AtatEnum}, AtatLen, serde_at::serde::Serialize};

#[derive(Clone, AtatEnum)]
pub enum WorkingMode {
    LoRaP2P = 0,
    LoRaWan = 1,
    FskP2P = 2,
}

#[derive(Clone)]
pub enum Bandwidth {
    LoRa125KHz,
    LoRa250KHz,
    LoRa500KHz,
    LoRa7_8MHz,
    LoRa10_4MHz,
    LoRa15_63MHz,
    LoRa20_83MHz,
    LoRa31_25MHz,
    LoRa41_67MHz,
    LoRa62_5MHz,
    FSK(u32),
}

impl AtatLen for Bandwidth {
    const LEN: usize = 6;
}

impl Serialize for Bandwidth {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: atat::serde_at::serde::Serializer {
        match self {
            Self::LoRa125KHz => serializer.serialize_str("0"),
            Self::LoRa250KHz => serializer.serialize_str("1"),
            Self::LoRa500KHz => serializer.serialize_str("2"),
            Self::LoRa7_8MHz => serializer.serialize_str("3"),
            Self::LoRa10_4MHz => serializer.serialize_str("4"),
            Self::LoRa15_63MHz => serializer.serialize_str("5"),
            Self::LoRa20_83MHz => serializer.serialize_str("6"),
            Self::LoRa31_25MHz => serializer.serialize_str("7"),
            Self::LoRa41_67MHz => serializer.serialize_str("8"),
            Self::LoRa62_5MHz => serializer.serialize_str("9"),
            Self::FSK(bw) => serializer.serialize_str(alloc::format!("{bw}").as_str()),
        }
    }
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+NWM", NoResponse)]
pub struct SetNetworkWorkingMode {
    pub mode: WorkingMode,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+NWM=?", NoResponse)]
pub struct GetNetworkWorkingMode {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PFREQ", NoResponse)]
pub struct SetP2PFrequency {
    pub frequency: u32,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PFREQ=?", P2PFrequencyResponse)]
pub struct GetP2PFrequency {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PSF", NoResponse)]
pub struct SetP2PSpreadingFactor {
    pub spreading_factor: u8,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PSF=?", P2PSpreadingFactorResponse)]
pub struct GetP2PSpreadingFactor {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PBW", NoResponse)]
pub struct SetP2PBandwidth {
    pub bandwidth: Bandwidth,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PBW=?", NoResponse)]
pub struct GetP2PBandwidth {}



