package dev.argon.verilization.java_runtime;

import java.math.BigInteger;
import java.io.IOException;

public final class Int {
    private Int() {}

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
