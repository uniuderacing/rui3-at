//! A library to send and receive data via [RUI 3] radio modules.
//!
//! It implements functions such as:
//! - Send and receive data.
//! - Send and receive AT commands.
//! - Set and read radio configuration.
//!
//! [RUI 3]: https://docs.rakwireless.com/RUI3/
//!
//! # Usage
//!
//! ```toml
//! [dependencies]
//! rui3_radio = "0.1.0"
//! ```
//!
//! # Examples
//!
//! ```rust
//! use rui3_radio::Rui3Radio;
//! use atat::ClientBuilder;
//!
//!
//! ```

#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(rustdoc::all)]
// #![warn(missing_docs)]
//#![no_std]

use at::commands::p2p::Encrypted;

extern crate alloc;

pub mod at;

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
    pub encrypted: Encrypted,
    /// The encryption key used.
    pub encryption_key: atat::heapless::String<16>,
}

/// Default trait implementation for Configuration.
impl Default for Configuration {
    fn default() -> Self {
        println!("Default configuration used");
        Self {
            working_mode: at::commands::p2p::WorkingMode::LoRaP2P,
            frequency: 868_000_000,
            spreading_factor: 7,
            bandwidth: at::commands::p2p::Bandwidth::LoRa125KHz,
            code_rate: at::commands::p2p::CodeRate::PCR4_5,
            preamble_length: 8,
            tx_power: 14,
            encrypted: Encrypted::False,
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
        // Add \r\n to the end of the string.

        data.iter()
            .map(|b| alloc::format!("{b:02X}"))
            .for_each(|s| string_result.push_str(&s).unwrap());

        string_result.push_str("\r\n").unwrap();
        let send_command = at::commands::p2p::SendData {
            payload: string_result,
        }; // TODO: this loop here.
           // Disable RX.
        self.client.send(&at::commands::p2p::ReceiveData {
            window: at::commands::p2p::ReceiveWindow::StopListening,
        })?;

        // Send data.
        self.client.send_retry(&send_command)?;
        println!("Sending data: {:?}", data);
        // Re-enable RX.
        self.client.send_retry(&at::commands::p2p::ReceiveData {
            window: at::commands::p2p::ReceiveWindow::Continuous,
        })?;
        Ok(())
    }

    /// Receives data in countinous mode.
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
            window: at::commands::p2p::ReceiveWindow::StopListening,
        };

        // Enable RX.
        self.client.send(&receive_command)?;

        let receive_command = at::commands::p2p::ReceiveData {
            window: at::commands::p2p::ReceiveWindow::OnePacket,
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
        receiving_window: &at::commands::p2p::ReceiveWindow,
    ) -> Result<alloc::vec::Vec<u8>, nb::Error<atat::Error>> {
        match receiving_window {
            at::commands::p2p::ReceiveWindow::Milliseconds(millis) => {
                // Enable RX
                self.client.send(&at::commands::p2p::ReceiveData {
                    window: at::commands::p2p::ReceiveWindow::Milliseconds(*millis),
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

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets the receiving window.
    pub fn set_receiving_window(
        &mut self,
        receiving_window: at::commands::p2p::ReceiveWindow,
    ) -> Result<(), nb::Error<atat::Error>> {
        self.client.send(&at::commands::p2p::ReceiveData {
            window: receiving_window,
        })?;
        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    #[allow(clippy::missing_panics_doc)]
    /// URC polling function, returns data as a vec u8.
    ///
    /// This function is usually only called through the [`receive`] function.
    ///
    /// [`receive`]: #method.receive
    pub fn poll(&mut self) -> Result<alloc::vec::Vec<u8>, nb::Error<atat::Error>> {
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

                Ok(data)
            }
            Some(at::urc::URCMessages::PeerToPeerInfo { rssi, snr }) => {
                self.rssi = rssi;
                self.snr = snr;
                Ok(alloc::vec![])
            }
            None => Ok(alloc::vec![]),
        }
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets client to the desired configuration.
    ///
    /// Takes as parameter a `Configuration` struct and returns nothing.
    pub fn configure(
        &mut self,
        configuration: Configuration,
    ) -> Result<(), nb::Error<atat::Error>> {
        // Set the frequency.
        println!("Config starting");
        println!(
            "Trying to set the frequency to: {}",
            configuration.frequency
        );
        self.client.send(&at::commands::p2p::SetP2PFrequency {
            frequency: configuration.frequency,
        })?;

        // // Set the working mode.
        // println!(
        //     "Trying to set the working mode to: {:?}",
        //     configuration.working_mode
        // );
        // self.client
        //     .send(&at::commands::p2p::SetNetworkWorkingMode {
        //         mode: configuration.working_mode,
        //     })?;

        // Set the spreading factor.
        println!(
            "Trying to set the spreading factor to: {:?}",
            configuration.spreading_factor
        );
        self.client
            .send(&at::commands::p2p::SetP2PSpreadingFactor {
                spreading_factor: configuration.spreading_factor,
            })?;
        // Set the bandwidth.
        println!(
            "Trying to set the bandwidth to: {:?}",
            configuration.bandwidth
        );
        self.client.send(&at::commands::p2p::SetP2PBandwidth {
            bandwidth: configuration.bandwidth,
        })?;
        // Set the code rate.
        println!(
            "Trying to set the code rate to: {:?}",
            configuration.code_rate
        );
        self.client.send(&at::commands::p2p::SetCodeRate {
            code_rate: configuration.code_rate,
        })?;
        // Set the preamble length.
        println!(
            "Trying to set the preamble length to: {:?}",
            configuration.preamble_length
        );
        self.client.send(&at::commands::p2p::SetPreambleLength {
            preamble_length: configuration.preamble_length,
        })?;
        // Set the TX power.
        println!(
            "Trying to set the TX power to: {:?}",
            configuration.tx_power
        );
        self.client.send(&at::commands::p2p::SetTxPower {
            tx_power: configuration.tx_power,
        })?;
        // // Set the encryption mode.
        // println!(
        //     "Trying to set the encryption mode to: {:?}",
        //     configuration.encrypted
        // );
        // self.client.send(&at::commands::p2p::SetEncryptionMode {
        //     encryption: configuration.encrypted,
        // })?;
        // // Set the encryption key.
        // println!(
        //     "Trying to set the encryption key to: {:?}",
        //     configuration.encryption_key
        // );
        // self.client.send(&at::commands::p2p::SetEncryptionKey {
        //     encryption_key: configuration.encryption_key,
        // })?;

        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
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

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets the network working mode.
    pub fn set_network_working_mode(
        &mut self,
        working_mode: at::commands::p2p::WorkingMode,
    ) -> Result<(), nb::Error<atat::Error>> {
        // Set the working mode.
        self.client
            .send(&at::commands::p2p::SetNetworkWorkingMode { mode: working_mode })?;
        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Gets the network working mode.
    pub fn get_network_working_mode(
        &mut self,
    ) -> Result<at::commands::p2p::WorkingMode, nb::Error<atat::Error>> {
        // Get the network working mode.
        let working_mode = self
            .client
            .send(&at::commands::p2p::GetNetworkWorkingMode {})?;
        Ok(working_mode.mode)
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets the frequency.
    pub fn set_frequency(&mut self, frequency: u32) -> Result<(), nb::Error<atat::Error>> {
        // Set the frequency.
        self.client
            .send(&at::commands::p2p::SetP2PFrequency { frequency })?;
        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Gets the frequency.
    pub fn get_frequency(&mut self) -> Result<u32, nb::Error<atat::Error>> {
        // Get the frequency.
        let frequency = self.client.send(&at::commands::p2p::GetP2PFrequency {})?;
        Ok(frequency.frequency)
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets the spreading factor.
    pub fn set_spreading_factor(
        &mut self,
        spreading_factor: u8,
    ) -> Result<(), nb::Error<atat::Error>> {
        // Set the spreading factor.
        self.client
            .send(&at::commands::p2p::SetP2PSpreadingFactor { spreading_factor })?;
        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Gets the spreading factor.
    pub fn get_spreading_factor(&mut self) -> Result<u8, nb::Error<atat::Error>> {
        // Get the spreading factor.
        let spreading_factor = self
            .client
            .send(&at::commands::p2p::GetP2PSpreadingFactor {})?;
        Ok(spreading_factor.spreading_factor)
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets the bandwidth.
    pub fn set_bandwidth(
        &mut self,
        bandwidth: at::commands::p2p::Bandwidth,
    ) -> Result<(), nb::Error<atat::Error>> {
        // Set the bandwidth.
        self.client
            .send(&at::commands::p2p::SetP2PBandwidth { bandwidth })?;
        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Gets the bandwidth.
    pub fn get_bandwidth(
        &mut self,
    ) -> Result<at::commands::p2p::Bandwidth, nb::Error<atat::Error>> {
        // Get the bandwidth.
        let bandwidth = self.client.send(&at::commands::p2p::GetP2PBandwidth {})?;
        Ok(bandwidth.bandwidth)
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets the code rate.
    pub fn set_code_rate(
        &mut self,
        code_rate: at::commands::p2p::CodeRate,
    ) -> Result<(), nb::Error<atat::Error>> {
        // Set the code rate.
        self.client
            .send(&at::commands::p2p::SetCodeRate { code_rate })?;
        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Gets the code rate.
    pub fn get_code_rate(&mut self) -> Result<at::commands::p2p::CodeRate, nb::Error<atat::Error>> {
        // Get the code rate.
        let code_rate = self.client.send(&at::commands::p2p::GetCodeRate {})?;
        Ok(code_rate.code_rate)
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets the preamble length.
    pub fn set_preamble_length(
        &mut self,
        preamble_length: u16,
    ) -> Result<(), nb::Error<atat::Error>> {
        // Set the preamble length.
        self.client
            .send(&at::commands::p2p::SetPreambleLength { preamble_length })?;
        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Gets the preamble length.
    pub fn get_preamble_length(&mut self) -> Result<u16, nb::Error<atat::Error>> {
        // Get the preamble length.
        let preamble_length = self.client.send(&at::commands::p2p::GetPreambleLength {})?;
        Ok(preamble_length.preamble_length)
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets tx power.
    pub fn set_tx_power(&mut self, tx_power: u8) -> Result<(), nb::Error<atat::Error>> {
        // Set tx power.
        self.client
            .send(&at::commands::p2p::SetTxPower { tx_power })?;
        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Gets tx power.
    pub fn get_tx_power(&mut self) -> Result<u8, nb::Error<atat::Error>> {
        // Get tx power.
        let tx_power = self.client.send(&at::commands::p2p::GetTxPower {})?;
        Ok(tx_power.tx_power)
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets the encryption mode.
    pub fn set_encryption_mode(&mut self, encryption: Encrypted) -> Result<(), nb::Error<atat::Error>> {
        // Set the encryption mode.
        self.client
            .send(&at::commands::p2p::SetEncryptionMode { encryption })?;
        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Gets the encryption mode.
    pub fn get_encryption_mode(&mut self) -> Result<Encrypted, nb::Error<atat::Error>> {
        // Get the encryption mode.
        let encryption = self.client.send(&at::commands::p2p::GetEncryptionMode {})?;
        Ok(encryption.encryption)
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Sets the encryption key.
    pub fn set_encryption_key(
        &mut self,
        encryption_key: atat::heapless::String<16>,
    ) -> Result<(), nb::Error<atat::Error>> {
        // Set the encryption key.
        self.client
            .send(&at::commands::p2p::SetEncryptionKey { encryption_key })?;
        Ok(())
    }

    #[allow(missing_doc_code_examples)]
    #[allow(clippy::missing_errors_doc)]
    /// Gets the encryption key.
    pub fn get_encryption_key(
        &mut self,
    ) -> Result<atat::heapless::String<16>, nb::Error<atat::Error>> {
        // Get the encryption key.
        let encryption_key = self.client.send(&at::commands::p2p::GetEncryptionKey {})?;
        Ok(encryption_key.encryption_key)
    }
}
