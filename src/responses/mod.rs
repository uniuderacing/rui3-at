use atat::atat_derive::AtatResp;

pub(crate) mod general;

#[derive(Clone, AtatResp)]
pub struct NoResponse {}