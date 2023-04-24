use atat::atat_derive::AtatResp;

pub mod general;
pub mod p2p;

#[derive(Clone, AtatResp)]
pub struct NoResponse {}
