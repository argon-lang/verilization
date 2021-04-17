use crate::{FormatReader, FormatWriter};
use num_bigint::{ BigUint, BigInt, Sign };
use num_traits::identities::{Zero, One};

pub fn encode_vlq<W : FormatWriter>(writer: &mut W, sign: Option<Sign>, n: &BigUint) -> Result<(), W::Error> {

    let mut n = n.clone();
    match sign {
        Some(Sign::Minus) => {
            n -= BigUint::one();
        },
        _ => (),
    }


    struct EncodeState {
        out_bit_index: u32,
        current_byte: u8,
    }

    fn put_bit<W : FormatWriter>(writer: &mut W, state: &mut EncodeState, b: bool) -> Result<(), W::Error> {
        if state.out_bit_index > 6 {
            writer.write_u8(state.current_byte | 0x80)?;
            state.out_bit_index = 0;
            state.current_byte = 0;
        }

        if b {
            state.current_byte |= 1 << state.out_bit_index;
        }
        state.out_bit_index += 1;
        Ok(())
    }


    let mut state = EncodeState {
        out_bit_index: 0,
        current_byte: 0,
    };

    for i in 0..n.bits() {
        put_bit(writer, &mut state, n.bit(i))?;
    }

    if let Some(sign) = sign {
        let neg = match sign {
            Sign::Minus => true,
            _ => false,
        };

        while state.out_bit_index != 6 {
            put_bit(writer, &mut state, false)?;
        }

        put_bit(writer, &mut state, neg)?;
    }

    writer.write_u8(state.current_byte)?;

    Ok(())
}

pub fn decoede_vlq_signed<R : FormatReader>(reader: &mut R) -> Result<BigInt, R::Error> {
    let mut b = reader.read_u8()?;
    let mut i: u64 = 0;
    let mut n = BigUint::zero();

    while (b & 0x80) != 0 {
        for j in 0..7 {
            n.set_bit(i, (b & (1 << j)) != 0);
            i += 1;
        }
        b = reader.read_u8()?;
    }

    for j in 0..6 {
        n.set_bit(i, (b & (1 << j)) != 0);
        i += 1;
    }

    let signbit = (b & 0x40) != 0;
    let sign = if signbit { Sign::Minus } else { Sign::Plus };

    if signbit {
        n -= BigUint::one();
    }

    Ok(BigInt::from_biguint(sign, n))
}

pub fn decoede_vlq_unsigned<R : FormatReader>(reader: &mut R) -> Result<BigUint, R::Error> {
    let mut b = reader.read_u8()?;
    let mut i: u64 = 0;
    let mut n = BigUint::zero();

    while (b & 0x80) != 0 {
        for j in 0..7 {
            n.set_bit(i, (b & (1 << j)) != 0);
            i += 1;
        }
        b = reader.read_u8()?;
    }

    for j in 0..7 {
        n.set_bit(i, (b & (1 << j)) != 0);
        i += 1;
    }

    Ok(n)
}
