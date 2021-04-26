package dev.argon.verilization.java_runtime;

import java.math.BigInteger;
import java.io.IOException;

public final class Nat {
    private Nat() {}

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
