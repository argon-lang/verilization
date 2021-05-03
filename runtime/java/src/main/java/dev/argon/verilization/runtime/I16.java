package dev.argon.verilization.runtime;

import java.io.IOException;

public final class I16 {
    private I16() {}

    public static short fromInteger(int i) {
        return (short)i;
    }

    public static final Codec<Short> codec = new Codec<Short>() {
        @Override
        public Short read(FormatReader reader) throws IOException {
            return reader.readShort();
        }

        @Override
        public void write(FormatWriter writer, Short value) throws IOException {
            writer.writeShort(value);
        }
    };
}
