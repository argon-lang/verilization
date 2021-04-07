mod vlq;

use num_bigint::{ BigUint, BigInt };
use num_traits::ToPrimitive;


pub trait FormatReader {
    type Error;
    fn read_u8(&mut self) -> Result<u8, Self::Error>;
    fn read_i8(&mut self) -> Result<i8, Self::Error>;
    fn read_u16(&mut self) -> Result<u16, Self::Error>;
    fn read_i16(&mut self) -> Result<i16, Self::Error>;
    fn read_u32(&mut self) -> Result<u32, Self::Error>;
    fn read_i32(&mut self) -> Result<i32, Self::Error>;
    fn read_u64(&mut self) -> Result<u64, Self::Error>;
    fn read_i64(&mut self) -> Result<i64, Self::Error>;
    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, Self::Error>;
}

pub trait FormatWriter {
    type Error;
    fn write_u8(&mut self, value: u8) -> Result<(), Self::Error>;
    fn write_i8(&mut self, value: i8) -> Result<(), Self::Error>;
    fn write_u16(&mut self, value: u16) -> Result<(), Self::Error>;
    fn write_i16(&mut self, value: i16) -> Result<(), Self::Error>;
    fn write_u32(&mut self, value: u32) -> Result<(), Self::Error>;
    fn write_i32(&mut self, value: i32) -> Result<(), Self::Error>;
    fn write_u64(&mut self, value: u64) -> Result<(), Self::Error>;
    fn write_i64(&mut self, value: i64) -> Result<(), Self::Error>;
    fn write_bytes(data: &Vec<u8>) -> Result<(), Self::Error>;
}

pub trait VerilizationCodec where Self : Sized {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error>;
    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error>;
}


impl VerilizationCodec for BigUint {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        vlq::decoede_vlq_unsigned(reader)
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        vlq::encode_vlq(writer, None, self)
    }
}

impl VerilizationCodec for BigInt {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        vlq::decoede_vlq_signed(reader)
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        vlq::encode_vlq(writer, Some(self.sign()), self.magnitude())
    }
}

impl VerilizationCodec for u8 {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        reader.read_u8()
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_u8(*self)
    }
}

impl VerilizationCodec for i8 {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        reader.read_i8()
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_i8(*self)
    }
}

impl VerilizationCodec for u16 {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        reader.read_u16()
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_u16(*self)
    }
}

impl VerilizationCodec for i16 {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        reader.read_i16()
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_i16(*self)
    }
}

impl VerilizationCodec for u32 {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        reader.read_u32()
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_u32(*self)
    }
}

impl VerilizationCodec for i32 {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        reader.read_i32()
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_i32(*self)
    }
}

impl VerilizationCodec for u64 {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        reader.read_u64()
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_u64(*self)
    }
}

impl VerilizationCodec for i64 {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        reader.read_i64()
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        writer.write_i64(*self)
    }
}

impl <T: VerilizationCodec> VerilizationCodec for Vec<T> {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        let len = BigUint::read_verilization(reader)?.to_usize().unwrap();
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(T::read_verilization(reader)?);
        }

        Ok(vec)
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        BigUint::from(self.len()).write_verilization(writer)?;
        for elem in self {
            elem.write_verilization(writer)?;
        }

        Ok(())
    }
}

impl <T: VerilizationCodec> VerilizationCodec for Option<T> {
    fn read_verilization<R : FormatReader>(reader: &mut R) -> Result<Self, R::Error> {
        let b = reader.read_i8()?;
        if b != 0 {
            Ok(Some(T::read_verilization(reader)?))
        }
        else {
            Ok(None)
        }
    }

    fn write_verilization<W : FormatWriter>(&self, writer: &mut W) -> Result<(), W::Error> {
        if let Some(value) = self {
            writer.write_u8(1)?;
            value.write_verilization(writer)?;
        }
        else {
            writer.write_u8(0)?;
        }
        Ok(())
    }
}



