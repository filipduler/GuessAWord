use std::io::Error;
use std::ops::Deref;
use anyhow::bail;
use tokio_util::bytes::{Bytes, BytesMut};
use crate::server::TcpMessage;

pub struct BinReader<'a> {
    buffer: &'a [u8],
    position: usize
}

impl<'a> BinReader<'a> {
    pub fn from_result(data: &'a Option<Result<BytesMut, std::io::Error>>) -> anyhow::Result<Self> {
        match data {
            Some(Ok(data)) =>
                Ok(Self {
                    buffer: data.deref(),
                    position: 0
                }),
            _ => bail!("invalid result")
        }
    }

    pub fn from_bytes(bytes: &'a Bytes) -> Self {
        Self {
            buffer: bytes.deref(),
            position: 0
        }
    }

    pub fn read_u8(&mut self) -> u8 {
        let value = self.buffer[self.position];
        self.position += 1;

        value
    }

    pub fn read_u16(&mut self) -> u16 {
        let bytes = &self.buffer[self.position..self.position + 2];
        self.position += 2;
        u16::from_le_bytes([bytes[0], bytes[1]])
    }

    pub fn read_u32(&mut self) -> u32 {
        let bytes = &self.buffer[self.position..self.position + 4];
        self.position += 4;
        u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
    }

    pub fn read_str(&mut self) -> String {
        let len = self.read_u16() as usize; // Read string length
        let bytes = &self.buffer[self.position..self.position + len];
        self.position += len;

        String::from_utf8(bytes.to_vec()).expect("Invalid UTF-8")
    }
}