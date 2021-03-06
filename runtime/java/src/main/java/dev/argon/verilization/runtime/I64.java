package dev.argon.verilization.runtime;

import java.io.IOException;

public final class I64 {
    private I64() {}

    public static long fromInteger(int i) {
        return i;
    }

    public static long fromInteger(long l) {
        return l;
    }

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
