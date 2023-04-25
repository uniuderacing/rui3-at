//! A library to send and receive data via [RUI 3] radio modules.
//!
//! It implements functions such as:
//! - Send and receive data.
//! - Send and receive AT commands.
//! - Set and read radio configuration.
//!
//! [RUI 3]: https://docs.rakwireless.com/RUI3/

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(rustdoc::all)]
#![warn(missing_docs)]
#![no_std]

extern crate alloc;

mod at;

/// A struct to define the radio client.
pub struct Rui3Radio<C>
where
    C: atat::AtatClient,
{
    client: C,
    rssi: i16,
    snr: i16,
}

/// A struct to define the radio configuration.
pub struct Configuration {
    /// The working mode of the radio.
    pub working_mode: at::commands::p2p::WorkingMode,
    /// The frequency used.
    pub frequency: u32,
    /// The spreading factor used.
    pub spreading_factor: u8,
    /// The bandwidth used.
    pub bandwidth: at::commands::p2p::Bandwidth,
    /// The code rate used.
    pub code_rate: at::commands::p2p::CodeRate,
    /// The preamble length used.
    pub preamble_length: u16,
    /// The TX power used.
    pub tx_power: u8,
    /// Whether the encryption is enabled or not.
    pub encrypted: bool,
    /// The encryption key used.
    pub encryption_key: atat::heapless::String<16>,
}

/// Default trait implementation for Configuration.
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
    /// Creates a new radio client.
    /// 
    /// # Arguments
    /// 
    /// * `client` - The AT client.
    /// 
    /// # Example
    /// 
    /// ```compile_fail
    /// let (mut client, ingress) = ClientBuilder::new(tx, timer, atat::Config::new(atat::Mode::Timeout)).build(queues);
    /// let radio_client = Rui3Radio::new(client);
    /// ```
    pub const fn new(client: C) -> Self {
        Self {
            client,
            rssi: 0,
            snr: 0,
        }
    }

    /// Converts data to hex and sends it.
    /// 
    /// Takes as parameter a slice of u8 and converts it to a hex string.
    /// Then temporarily disables RX and sends the data.
    /// Finally re-enables RX.
    /// 
    /// # Arguments
    /// 
    /// * `data` - The data to send.
    /// 
    /// # Example
    /// 
    /// ```compile_fail
    /// let data = [5; 4];
    /// radio_client.send(&data)?;
    /// ```
    /// 
    /// # Errors
    /// 
    /// TODO
    /// 
    /// # Panics
    /// 
    /// TODO
    pub fn send(&mut self, data: &[u8]) -> Result<(), nb::Error<atat::Error>> {
        // Convert each byte of data to a hex string.
        let mut string_result: atat::heapless::String<500> = "".into();

        data.iter()
            .map(|b| alloc::format!("{b:02X}"))
            .for_each(|s| string_result.push_str(&s).unwrap());

        let send_command = at::commands::p2p::SendData {
            payload: string_result,
        };

        // Disable RX.
        self.client.send(&at::commands::p2p::ReceiveData {
            window: at::commands::p2p::ReceiveWindow::StopListening,
        })?;

        // Send data.
        self.client.send(&send_command)?;

        // Re-enable RX.
        self.client.send(&at::commands::p2p::ReceiveData {
            window: at::commands::p2p::ReceiveWindow::Continuous,
        })?;

        Ok(())
    }

    /// Receives data in countinuous mode.
    /// 
    /// Checks for URCs in a loop and returns the data as a vector of u8.
    /// If configured in RX mode, any new values of AT+PRECV will not be accepted.
    /// To stop receiving, send AT+PRECV=0 via TODO
    /// 
    /// # Example
    /// 
    /// ```compile_fail
    /// let data = radio_client.receive()?;
    /// ```
    /// 
    /// # Errors
    /// 
    /// TODO
    /// 
    /// # Panics
    /// 
    /// TODO
    pub fn receive(&mut self) -> Result<alloc::vec::Vec<u8>, nb::Error<atat::Error>> {
        // Recieve is blocking until data is received.

        let receive_command = at::commands::p2p::ReceiveData {
            window: at::commands::p2p::ReceiveWindow::Continuous,
        };

        // Enable RX.
        self.client.send(&receive_command)?;

        loop {
            // Check for URCs in loop.
            let check_urc = self.client.check_urc::<at::urc::URCMessages>();

            match check_urc {
                Some(at::urc::URCMessages::PeerToPeerData(hex_data)) => {
                    // let data = decode(hex_data).unwrap();

                    // if hex_data.len() % 2 != 0 {
                    //     Err
                    // }

                    let hex_data = alloc::string::String::from_utf8(hex_data).unwrap();
                    let mut data = alloc::vec::Vec::new();
                    for i in (0..hex_data.len()).step_by(2) {
                        let byte = u8::from_str_radix(&hex_data[i..i + 2], 16).unwrap();
                        data.push(byte);
                    }

                    // Return as a vector of u8.
                    return Ok(data);
                }
                Some(at::urc::URCMessages::PeerToPeerInfo { rssi, snr }) => {
                    self.rssi = rssi;
                    self.snr = snr;
                }
                None => {
                    return Ok(alloc::vec![]);
                }
            }
        }
    }

    /// Receives data in any mode.
    /// 
    /// Takes as parameter a `ReceiveWindow` enum and returns the data as a vector of u8.
    /// Possible values are:
    /// * `ReceiveWindow::Milliseconds(millis)` - Receives data for a certain amount of time.
    /// * `ReceiveWindow::OnePacket` - Receives data for one packet.
    /// * `ReceiveWindow::Continuous` - Receives data in continuous mode.
    /// * `ReceiveWindow::StopListening` - Stops listening.
    /// 
    /// # Arguments
    /// 
    /// * `receiving_window` - The receiving window.
    /// 
    /// # Example
    /// 
    /// ```compile_fail
    /// let data = radio_client.receive_explicit(at::commands::p2p::ReceiveWindow::Milliseconds(millis))?;
    /// ```
    /// 
    /// ```compile_fail
    /// let data = radio_client.receive_explicit(at::commands::p2p::ReceiveWindow::OnePacket)?;
    /// ```
    /// 
    /// # Errors
    /// 
    /// TODO
    /// 
    /// # Panics
    /// 
    /// TODO
    pub fn receive_explicit(
        &mut self,
        receiving_window: at::commands::p2p::ReceiveWindow,
    ) -> Result<alloc::vec::Vec<u8>, nb::Error<atat::Error>> {
        match receiving_window {
            at::commands::p2p::ReceiveWindow::Milliseconds(millis) => {
                // Enable RX
                self.client.send(&at::commands::p2p::ReceiveData {
                    window: at::commands::p2p::ReceiveWindow::Milliseconds(millis),
                })?;

                // TODO: find how to wait for a certain amount of time.

                let data: alloc::vec::Vec<u8> = alloc::vec![];
                Ok(data)
            }
            at::commands::p2p::ReceiveWindow::OnePacket => {
                // Enable RX
                self.client.send(&at::commands::p2p::ReceiveData {
                    window: at::commands::p2p::ReceiveWindow::OnePacket,
                })?;

                loop {
                    // Check for URCs in loop.
                    let check_urc = self.client.check_urc::<at::urc::URCMessages>();

                    match check_urc {
                        Some(at::urc::URCMessages::PeerToPeerData(hex_data)) => {
                            // let data = decode(hex_data).unwrap();

                            // if hex_data.len() % 2 != 0 {
                            //     Err
                            // }

                            let hex_data = alloc::string::String::from_utf8(hex_data).unwrap();
                            let mut data = alloc::vec::Vec::new();
                            for i in (0..hex_data.len()).step_by(2) {
                                let byte = u8::from_str_radix(&hex_data[i..i + 2], 16).unwrap();
                                data.push(byte);
                            }

                            // Return as a vector of u8.
                            return Ok(data);
                        }
                        Some(at::urc::URCMessages::PeerToPeerInfo { rssi, snr }) => {
                            self.rssi = rssi;
                            self.snr = snr;
                        }
                        None => {
                            continue;
                        }
                    }
                }
            }
            at::commands::p2p::ReceiveWindow::Continuous => self.receive(),
            at::commands::p2p::ReceiveWindow::StopListening => {
                // Disable RX
                self.client.send(&at::commands::p2p::ReceiveData {
                    window: at::commands::p2p::ReceiveWindow::StopListening,
                })?;
                Ok(alloc::vec![])
            }
        }
    }

    /// Sets client to the desired configuration.
    /// 
    /// Takes as parameter a `Configuration` struct and returns nothing.
    pub fn configure(
        &mut self,
        configuration: Configuration,
    ) -> Result<(), nb::Error<atat::Error>> {
        // Set the working mode.
        self.client
            .send(&at::commands::p2p::SetNetworkWorkingMode {
                mode: configuration.working_mode,
            })?;
        // Set the frequency.
        self.client.send(&at::commands::p2p::SetP2PFrequency {
            frequency: configuration.frequency,
        })?;
        // Set the spreading factor.
        self.client
            .send(&at::commands::p2p::SetP2PSpreadingFactor {
                spreading_factor: configuration.spreading_factor,
            })?;
        // Set the bandwidth.
        self.client.send(&at::commands::p2p::SetP2PBandwidth {
            bandwidth: configuration.bandwidth,
        })?;
        // Set the code rate.
        self.client.send(&at::commands::p2p::SetCodeRate {
            code_rate: configuration.code_rate,
        })?;
        // Set the preamble length.
        self.client.send(&at::commands::p2p::SetPreambleLength {
            preamble_length: configuration.preamble_length,
        })?;
        // Set the TX power.
        self.client.send(&at::commands::p2p::SetTxPower {
            tx_power: configuration.tx_power,
        })?;
        // Set the encryption key.
        self.client.send(&at::commands::p2p::SetEncryptionKey {
            encryption_key: configuration.encryption_key,
        })?;
        // Set the encryption mode.
        self.client.send(&at::commands::p2p::SetEncryptionMode {
            encryption: configuration.encrypted,
        })?;

        Ok(())
    }

    /// Reads client configuration and returns a `Configuration` struct.
    pub fn read_configuration(&mut self) -> Result<Configuration, nb::Error<atat::Error>> {
        // Get the network working mode.
        let working_mode = self
            .client
            .send(&at::commands::p2p::GetNetworkWorkingMode {})?;

        // Get the frequency.
        let frequency = self.client.send(&at::commands::p2p::GetP2PFrequency {})?;
        // Get the spreading factor.
        let spreading_factor = self
            .client
            .send(&at::commands::p2p::GetP2PSpreadingFactor {})?;
        // Get the bandwidth.
        let bandwidth = self.client.send(&at::commands::p2p::GetP2PBandwidth {})?;
        // Get the code rate.
        let code_rate = self.client.send(&at::commands::p2p::GetCodeRate {})?;
        // Get the preamble length.
        let preamble_length = self.client.send(&at::commands::p2p::GetPreambleLength {})?;
        // Get the TX power.
        let tx_power = self.client.send(&at::commands::p2p::GetTxPower {})?;
        // Get the encryption key.
        let encryption_key = self.client.send(&at::commands::p2p::GetEncryptionKey {})?;
        // Get the encryption mode.
        let encryption_mode = self.client.send(&at::commands::p2p::GetEncryptionMode {})?;

        let configuration = Configuration {
            working_mode: working_mode.mode,
            frequency: frequency.frequency,
            spreading_factor: spreading_factor.spreading_factor,
            bandwidth: bandwidth.bandwidth,
            code_rate: code_rate.code_rate,
            preamble_length: preamble_length.preamble_length,
            tx_power: tx_power.tx_power,
            encrypted: encryption_mode.encryption,
            encryption_key: encryption_key.encryption_key,
        };

        // Return configuration
        Ok(configuration)
    }

    /// Returns the 'Received signal strength indicator' (RSSI) value.
    pub const fn get_rssi(&self) -> i16 {
        self.rssi
    }
    
    /// Returns the 'Signal to noise ratio' (SNR) value.
    pub const fn get_snr(&self) -> i16 {
        self.snr
    }
}
