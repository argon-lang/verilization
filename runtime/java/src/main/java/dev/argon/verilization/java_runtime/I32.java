package dev.argon.verilization.java_runtime;

import java.io.IOException;

public final class I32 {
    private I32() {}

    public static final Codec<Integer> codec = new Codec<Integer>() {
        @Override
        public Integer read(FormatReader reader) throws IOException {
            return reader.readInt();
        }

        @Override
        public void write(FormatWriter writer, Integer value) throws IOException {
            writer.writeInt(value);
        }
    };
}
