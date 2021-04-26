package dev.argon.verilization.java_runtime;

import java.io.IOException;

public final class I64 {
    private I64() {}

    public static final Codec<Long> codec = new Codec<Long>() {
        @Override
        public Long read(FormatReader reader) throws IOException {
            return reader.readLong();
        }

        @Override
        public void write(FormatWriter writer, Long value) throws IOException {
            writer.writeLong(value);
        }
    };
}
