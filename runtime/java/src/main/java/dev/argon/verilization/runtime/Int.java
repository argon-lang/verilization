package dev.argon.verilization.runtime;

import java.math.BigInteger;
import java.io.IOException;

public final class Int {
    private Int() {}

    public static BigInteger fromInteger(int i) {
        return BigInteger.valueOf(i);
    }

    public static BigInteger fromInteger(long l) {
        return BigInteger.valueOf(l);
    }

    public static BigInteger fromInteger(BigInteger i) {
        return i;
    }

    public static final Codec<BigInteger> intCodec = new Codec<BigInteger>() {
        @Override
        public BigInteger read(FormatReader reader) throws IOException {
            return VLQ.decodeVLQ(reader, true);
        }

        @Override
        public void write(FormatWriter writer, BigInteger value) throws IOException {
            VLQ.encodeVLQ(writer, true, value);
        }

    };
}
