package dev.argon.verilization.runtime;

import java.util.Optional;
import java.io.IOException;

public final class Option {
    private Option() {}

    public static <A> Optional<A> fromCaseSome(A value) {
        return Optional.of(value);
    }

    public static <A> Optional<A> fromCaseNone() {
        return Optional.empty();
    }

    public static <A, B> Converter<Optional<A>, Optional<B>> converter(Converter<A, B> elementConverter) {
        return prev -> prev.map(elementConverter::convert);
    }

    public static <T> Codec<Optional<T>> codec(Codec<T> elementCodec) {
        return new Codec<Optional<T>>() {
            @Override
            public Optional<T> read(FormatReader reader) throws IOException {
                boolean present = reader.readByte() != 0;
                if(present) {
                    return Optional.of(elementCodec.read(reader));
                }
                else {
                    return Optional.empty();
                }
            }
    
            @Override
            public void write(FormatWriter writer, Optional<T> value) throws IOException {
                if(value.isPresent()) {
                    writer.writeByte((byte)1);
                    elementCodec.write(writer, value.get());
                }
                else {
                    writer.writeByte((byte)0);
                }
            }
    
        };
    }
}
