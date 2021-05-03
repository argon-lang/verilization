package dev.argon.verilization.runtime;

import java.math.BigInteger;
import java.io.IOException;

public final class Nat {
    private Nat() {}

    public static BigInteger fromInteger(int i) {
        return BigInteger.valueOf(i).abs();
    }

    public static BigInteger fromInteger(long l) {
        return BigInteger.valueOf(l).abs();
    }

    public static BigInteger fromInteger(BigInteger i) {
        return i.abs();
    }

    public static final Codec<BigInteger> codec = new Codec<BigInteger>() {
        @Override
        public BigInteger read(FormatReader reader) throws IOException {
            return VLQ.decodeVLQ(reader, false);
        }

        @Override
        public void write(FormatWriter writer, BigInteger value) throws IOException {
            VLQ.encodeVLQ(writer, false, value);
        }

    };
}
