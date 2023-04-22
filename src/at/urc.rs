use alloc::string::String;
use alloc::vec::Vec;
use atat::AtatUrc;
use text_io::scan;

pub enum URCMessages {
    PeerToPeerData(Vec<u8>),
    PeerToPeerInfo { rssi: i16, snr: i16 },
}

impl AtatUrc for URCMessages {
    type Response = Self;

    fn parse(resp: &[u8]) -> Option<Self::Response> {
        if &resp[..=4] == b"+EVT:" {
            let status = String::from_utf8(resp[5..].to_vec()).unwrap();

            // Peer to peer info.
            if status.starts_with("RXP2P") {
                let rssi: i16;
                let snr: i16;

                scan!(status.bytes() => "RXP2P, RSSI {}, SNR {}", rssi, snr);

                return Some(Self::PeerToPeerInfo { rssi, snr });
            }

            // Peer to peer data.
            if status.chars().all(|c| "0123456789ABCDEF".contains(c)) && status.len() % 2 == 0 {
                // All the characters are hexadecimal.

                // Split characters two by two and convert them to bytes.
                let mut data = Vec::new();
                for i in (0..status.len()).step_by(2) {
                    let byte = u8::from_str_radix(&status[i..i + 2], 16).unwrap();
                    data.push(byte);
                }

                return Some(Self::PeerToPeerData(data));
            }
        }

        None
    }
}
