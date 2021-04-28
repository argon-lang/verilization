package dev.argon.verilization.java_runtime;

import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.math.BigInteger;

public final class String {
    private String() {}

    public static java.lang.String fromString(java.lang.String s) {
        return s;
    }

    public static final Codec<java.lang.String> codec = new Codec<java.lang.String>() {
        @Override
        public java.lang.String read(FormatReader reader) throws IOException {
            BigInteger length = Nat.codec.read(reader);
            byte[] data = reader.readBytes(length.intValueExact());
            return new java.lang.String(data, StandardCharsets.UTF_8);
        }

        @Override
        public void write(FormatWriter writer, java.lang.String value) throws IOException {
            byte[] data = value.getBytes(StandardCharsets.UTF_8);
            Nat.codec.write(writer, BigInteger.valueOf(data.length));
            writer.writeBytes(data);
        }
    };
}
