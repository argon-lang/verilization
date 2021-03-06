package dev.argon.verilization.runtime;

import java.io.IOException;

public final class I32 {
    private I32() {}

    public static int fromInteger(int i) {
        return i;
    }

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
