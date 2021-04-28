package dev.argon.verilization.java_runtime;

import java.io.IOException;

public final class I8 {
    private I8() {}

    public static byte fromInteger(int i) {
        return (byte)i;
    }

    public static final Codec<Byte> codec = new Codec<Byte>() {
        @Override
        public Byte read(FormatReader reader) throws IOException {
            return reader.readByte();
        }

        @Override
        public void write(FormatWriter writer, Byte value) throws IOException {
            writer.writeByte(value);
        }
    };
}
