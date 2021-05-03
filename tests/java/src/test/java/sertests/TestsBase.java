package sertests;

import java.io.IOException;
import java.nio.ByteBuffer;
import java.util.Arrays;

import dev.argon.verilization.runtime.Codec;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

public class TestsBase {
    protected <T> void check(Codec<T> codec, T value, byte[] encoded) throws IOException {
        {
            final var writer = new MemoryFormatWriter();
            codec.write(writer, value);
            assertArrayEquals(writer.toByteArray(), encoded);
        }

        {
            final var reader = new MemoryFormatReader(ByteBuffer.wrap(encoded));
            final var decoded = codec.read(reader);
            assertTrue(reader.isEOF());
            assertEquals(decoded, value);
            assertEquals(value, decoded);
        }
    }
}
