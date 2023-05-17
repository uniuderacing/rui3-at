use alloc::string::String;
use alloc::vec::Vec;
use atat::{AtatUrc, Parser};
use text_io::scan;

pub enum URCMessages {
    PeerToPeerData(Vec<u8>),
    PeerToPeerInfo { rssi: i16, snr: i16 },
    PeerToPeerMessage { rssi: i16, snr: i16, data: Vec<u8> },
}

impl AtatUrc for URCMessages {
    type Response = Self;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        let status = String::from_utf8(resp.to_vec()).unwrap();

        // Peer to peer info.
        if status.starts_with("RXP2P") {
            let rssi: i16;
            let snr: i16;
            let data_byte: i16;
            let mut data = Vec::new();
            let status_clone = status.clone();

            scan!(status_clone.bytes() => "RXP2P:{}:{}", rssi, snr);

            // TODO: Remove chars untill "snr:".
            
            // Peer to peer data.
            if status.chars().all(|c| "0123456789ABCDEF".contains(c)) && status.len() % 2 == 0 {
                // All the characters are hexadecimal.
    
                // Split characters two by two and convert them to bytes.
                for i in (0..status.len()).step_by(2) {
                    let byte = u8::from_str_radix(&status[i..i + 2], 16).unwrap();
                    data.push(byte);
                }
            }
            return Some(Self::PeerToPeerMessage{rssi, snr, data});
        }


        None
    }
}

impl Parser for URCMessages {
    fn parse(buf: &[u8]) -> core::result::Result<(&[u8], usize), atat::digest::ParseError> {
        if buf.len() < 5 {
            return Err(atat::digest::ParseError::NoMatch);
        }

        if &buf[..=4] == b"+EVT:" {
            Ok((&buf[5..], buf.len() - 5))
        } else {
            Err(atat::digest::ParseError::NoMatch)
        }
    }
}
