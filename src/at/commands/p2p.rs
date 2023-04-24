#[allow(clippy::wildcard_imports)]
use crate::at::responses::{p2p::*, NoResponse};
use atat::{
    atat_derive::{AtatCmd, AtatEnum},
    serde_at::serde::{Deserialize, Serialize},
    AtatLen,
};

#[derive(Clone, AtatEnum)]
pub enum WorkingMode {
    LoRaP2P = 0,
    LoRaWan = 1,
    FskP2P = 2,
}

#[derive(Clone, AtatEnum)]
pub enum CodeRate {
    PCR4_5 = 0,
    PCR4_6 = 1,
    PCR4_7 = 2,
    PCR4_8 = 3,
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
        S: atat::serde_at::serde::Serializer,
    {
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

impl<'a> Deserialize<'a> for Bandwidth {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: atat::serde_at::serde::Deserializer<'a>,
    {
        let s = atat::serde_at::serde::Deserialize::deserialize(deserializer)?;
        match s {
            "0" => Ok(Self::LoRa125KHz),
            "1" => Ok(Self::LoRa250KHz),
            "2" => Ok(Self::LoRa500KHz),
            "3" => Ok(Self::LoRa7_8MHz),
            "4" => Ok(Self::LoRa10_4MHz),
            "5" => Ok(Self::LoRa15_63MHz),
            "6" => Ok(Self::LoRa20_83MHz),
            "7" => Ok(Self::LoRa31_25MHz),
            "8" => Ok(Self::LoRa41_67MHz),
            "9" => Ok(Self::LoRa62_5MHz),
            _ => Ok(Self::FSK(s.parse().unwrap())),
        }
    }
}

#[derive(Clone)]
pub enum ReceiveWindow {
    Milliseconds(u16),
    OnePacket,
    Continuous,
    StopListening,
}

impl AtatLen for ReceiveWindow {
    const LEN: usize = 5;
}

impl Serialize for ReceiveWindow {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: atat::serde_at::serde::Serializer,
    {
        match self {
            Self::Milliseconds(ms) => serializer.serialize_str(alloc::format!("{ms}").as_str()),
            Self::OnePacket => serializer.serialize_str("65535"),
            Self::Continuous => serializer.serialize_str("65534"),
            Self::StopListening => serializer.serialize_str("0"),
        }
    }
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+NWM", NoResponse)]
pub struct SetNetworkWorkingMode {
    pub mode: WorkingMode,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+NWM=?", NetworkWorkingModeResponse)]
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
#[at_cmd("+PBW=?", P2PBandwidth)]
pub struct GetP2PBandwidth {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PCR", NoResponse)]
pub struct SetCodeRate {
    pub code_rate: CodeRate,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PCR=?", P2PCodeRate)]
pub struct GetCodeRate {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PPL", NoResponse)]
pub struct SetPreambleLength {
    pub preamble_length: u16,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PPL=?", P2PPreambleLength)]
pub struct GetPreambleLength {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PTP", NoResponse)]
pub struct SetTxPower {
    pub tx_power: u8,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PTP=?", P2PTxPower)]
pub struct GetTxPower {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PSEND", NoResponse)]
pub struct SendData {
    pub payload: atat::heapless::String<500>,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+PRECV", NoResponse)]
pub struct ReceiveData {
    pub window: ReceiveWindow,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+ENCRY", NoResponse)]
pub struct SetEncryptionMode {
    pub encryption: bool,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+ENCRY=?", P2PEncryptionMode)]
pub struct GetEncryptionMode {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+ENCKEY", NoResponse)]
pub struct SetEncryptionKey {
    pub encryption_key: atat::heapless::String<16>,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+ENCKEY=?", P2PEncryptionKey)]
pub struct GetEncryptionKey {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+P2P", NoResponse)]
pub struct SetP2P {
    pub frequency: u32,
    pub spreading_factor: u8,
    pub bandwidth: Bandwidth,
    pub code_rate: CodeRate,
    pub preamble_length: u16,
    pub tx_power: u8,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+P2P=?", P2Pparameters)]
pub struct GetP2P {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+IQINVER", NoResponse)]
pub struct SetIqInversion {
    pub iq_inversion: bool,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+IQINVER=?", P2PIQInversion)]
pub struct GetIqInversion {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+SYNCWORD", NoResponse)]
pub struct SetSyncWord {
    pub sync_word: atat::heapless::String<4>,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+SYNCWORD=?", P2PSyncWord)]
pub struct GetSyncWord {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+SYMBOLTIMEOUT", NoResponse)]
pub struct SetSymbolTimeout {
    pub symbol_timeout: u8,
}

#[derive(Clone, AtatCmd)]
#[at_cmd("+SYMBOLTIMEOUT=?", P2PSymbolTimeout)]
pub struct GetSymbolTimeout {}
