use crate::responses::general::*;
use crate::responses::NoResponse;
use atat::atat_derive::AtatCmd;

#[derive(Clone, AtatCmd)]
#[at_cmd("", NoResponse)]
pub struct Attention {}

#[derive(Clone, AtatCmd)]
#[at_cmd("Z", NoResponse)]
pub struct McuReset {}

#[derive(Clone, AtatCmd)]
#[at_cmd("R", NoResponse)]
pub struct RestoreDefaultParameters {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+SN=?", SerialNumberResponse)]
pub struct SerialNumber {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+VER=?", FirmwareVersionResponse)]
pub struct FirmwareVersion {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+HWMODEL=?", HardwareVersionResponse)]
pub struct HardwareModel {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+ALIAS", NoResponse)]
pub struct SetAlias {}

#[derive(Clone, AtatCmd)]
#[at_cmd("+ALIAS=?", AliasResponse)]
pub struct GetAlias {}
