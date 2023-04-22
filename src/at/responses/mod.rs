use atat::atat_derive::AtatResp;

pub(crate) mod general;
pub(crate) mod p2p;

#[derive(Clone, AtatResp)]
pub struct NoResponse {}
