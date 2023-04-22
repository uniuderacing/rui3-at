#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]
#![warn(clippy::cargo)]
#![warn(rustdoc::all)]
#![no_std]

use embedded_hal::serial::{Read, Write};

extern crate alloc;

mod at;

pub struct Rui3Radio<SERIAL>
where
    SERIAL: Read<u8> + Write<u8>,
{
    serial: SERIAL,
}

impl<SERIAL> Rui3Radio<SERIAL>
where
    SERIAL: Read<u8> + Write<u8>,
{
    pub fn new(serial: SERIAL) -> Self {
        Self { serial }
    }
}
