
pub struct BinWriter {
    buffer: Vec<u8>,
    position: usize
}

impl BinWriter {
    pub fn with_capacity(size: usize) -> Self {
        Self {
            buffer: vec![0; size],
            position: 0
        }
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buffer[self.position] = value;
        self.position += 1;
    }

    pub fn write_str(&mut self, str: &str) {
        let bytes = str.as_bytes();
        let len = bytes.len() as u16; // Assuming length fits in u16

        // Write the length as an u16 (2 bytes)
        self.write_u16(len);

        // Write the string bytes
        self.buffer[self.position..self.position + bytes.len()]
            .copy_from_slice(bytes);
        self.position += bytes.len();
    }

    pub fn write_u16(&mut self, value: u16) {
        self.buffer[self.position..self.position + 2]
            .copy_from_slice(&value.to_le_bytes());
        self.position += 2;
    }

    pub fn write_u32(&mut self, value: u32) {
        self.buffer[self.position..self.position + 4]
            .copy_from_slice(&value.to_le_bytes());
        self.position += 4;
    }

    pub fn clone_data(&self) -> Vec<u8> {
        self.buffer[..self.position].to_vec()
    }

    pub fn len(&self) -> usize {
        self.position
    }

    pub fn clear(&mut self) {
        self.position = 0;
    }
}