#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(rustdoc::all, missing_docs)]
#![no_std]

extern crate alloc;

mod at;

pub struct Rui3Radio<C>
where
    C: atat::AtatClient,
{
    client: C,
}

pub struct Configuration {
    pub working_mode: at::commands::p2p::WorkingMode,
    pub frequency: u32,
    pub spreading_factor: u8,
    pub bandwidth: at::commands::p2p::Bandwidth,
    pub code_rate: at::commands::p2p::CodeRate,
    pub preamble_length: u16,
    pub tx_power: u8,
    pub encrypted: bool,
    pub encryption_key: atat::heapless::String<16>,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            working_mode: at::commands::p2p::WorkingMode::LoRaP2P,
            frequency: 868_000_000,
            spreading_factor: 7,
            bandwidth: at::commands::p2p::Bandwidth::LoRa125KHz,
            code_rate: at::commands::p2p::CodeRate::PCR4_5,
            preamble_length: 8,
            tx_power: 14,
            encrypted: false,
            encryption_key: "".into(),
        }
    }
}

impl<C> Rui3Radio<C>
where
    C: atat::AtatClient,
{
    pub fn new(client: C) -> Self {
        Self { client }
    }

    pub fn send(&self, data: &[u8]) -> Result<(), atat::Error> {
        // Convert each byte of data to a hex string.
        let data = data
            .iter()
            .map(|b| alloc::format!("{:02X}", b))
            .collect::<alloc::vec::Vec<atat::heapless::String>>()
            .join("");

        let send_command = at::commands::p2p::SendData { payload: data };

        // TODO: Disable RX.

        self.client.send(&send_command)?;

        // TODO: Re-enable RX.

        Ok(())
    }

    pub fn receive(
        &self,
    ) -> Result<alloc::vec::Vec<u8>, atat::Error> {
        let receive_command = at::commands::p2p::ReceiveData { window: at::commands::p2p::ReceiveWindow::Continuous };

        // Enable RX.

        self.client.send(&receive_command)?;

        // Check for URCs in loop.
        self.client.check_urc();

        // Decode data from hex and concatenate responses in an array.

        // Return as a vector of u8.

        Ok()
    }

    pub fn configure(&self, configuration: Configuration) {

    }

    // A function that reads the configuration.
    
}
