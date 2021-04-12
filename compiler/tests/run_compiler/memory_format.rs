use verilization_compiler::lang::GeneratorError;

pub struct MemoryFormatWriter {
    data: Vec<u8>,
}

impl MemoryFormatWriter {
    pub fn new() -> MemoryFormatWriter {
        MemoryFormatWriter {
            data: Vec::new(),
        }
    }

    pub fn data(self) -> Vec<u8> {
        self.data
    }
}

impl verilization_runtime::FormatWriter for MemoryFormatWriter {
    type Error = GeneratorError;
    fn write_u8(&mut self, value: u8) -> Result<(), Self::Error> {
        self.data.push(value);
        Ok(())
    }

    fn write_u16(&mut self, value: u16) -> Result<(), Self::Error> {
        self.data.push(value as u8);
        self.data.push((value >> 8) as u8);
        Ok(())
    }

    fn write_u32(&mut self, value: u32) -> Result<(), Self::Error> {
        self.write_u16(value as u16)?;
        self.write_u16((value >> 16) as u16)
    }

    fn write_u64(&mut self, value: u64) -> Result<(), Self::Error> {
        self.write_u32(value as u32)?;
        self.write_u32((value >> 32) as u32)
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.data.extend(data);
        Ok(())
    }
}
