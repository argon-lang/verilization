package dev.argon.verilization.java_runtime;

import java.io.IOException;
import java.math.BigInteger;
import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.ArrayList;
import java.util.Optional;

public abstract class StandardCodecs {
    private StandardCodecs() {}


    public static final Codec<BigInteger> natCodec = new Codec<BigInteger>() {

        @Override
        public BigInteger read(FormatReader reader) throws IOException {
            return VLQ.decodeVLQ(reader, false);
        }

        @Override
        public void write(FormatWriter writer, BigInteger value) throws IOException {
            VLQ.encodeVLQ(writer, false, value);
        }

    };

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

    public static final Codec<Byte> i8Codec = new Codec<Byte>() {

        @Override
        public Byte read(FormatReader reader) throws IOException {
            return reader.readByte();
        }

        @Override
        public void write(FormatWriter writer, Byte value) throws IOException {
            writer.writeByte(value);
        }

    };

    public static final Codec<Short> i16Codec = new Codec<Short>() {

        @Override
        public Short read(FormatReader reader) throws IOException {
            return reader.readShort();
        }

        @Override
        public void write(FormatWriter writer, Short value) throws IOException {
            writer.writeShort(value);
        }

    };

    public static final Codec<Integer> i32Codec = new Codec<Integer>() {

        @Override
        public Integer read(FormatReader reader) throws IOException {
            return reader.readInt();
        }

        @Override
        public void write(FormatWriter writer, Integer value) throws IOException {
            writer.writeInt(value);
        }

    };

    public static final Codec<Long> i64Codec = new Codec<Long>() {

        @Override
        public Long read(FormatReader reader) throws IOException {
            return reader.readLong();
        }

        @Override
        public void write(FormatWriter writer, Long value) throws IOException {
            writer.writeLong(value);
        }

    };

    public static final Codec<String> stringCodec = new Codec<String>() {

        @Override
        public String read(FormatReader reader) throws IOException {
            BigInteger length = natCodec.read(reader);
            if(length.compareTo(BigInteger.valueOf(Integer.MAX_VALUE)) > 0) {
                throw new ArithmeticException("Length of string would overflow");
            }

            byte[] data = reader.readBytes(length.intValue());
            return new String(data, StandardCharsets.UTF_8);
        }

        @Override
        public void write(FormatWriter writer, String value) throws IOException {
            byte[] data = value.getBytes(StandardCharsets.UTF_8);
            natCodec.write(writer, BigInteger.valueOf(data.length));
            writer.writeBytes(data);
        }

    };

    public static final Codec<byte[]> i8ListCodec = new Codec<byte[]>() {

        @Override
        public byte[] read(FormatReader reader) throws IOException {
            BigInteger length = natCodec.read(reader);
            if(length.compareTo(BigInteger.valueOf(Integer.MAX_VALUE)) > 0) {
                throw new ArithmeticException("Length of array would overflow");
            }

            return reader.readBytes(length.intValue());
        }

        @Override
        public void write(FormatWriter writer, byte[] value) throws IOException {
            natCodec.write(writer, BigInteger.valueOf(value.length));
            writer.writeBytes(value);
        }

    };

    public static final Codec<short[]> i16ListCodec = new Codec<short[]>() {

        @Override
        public short[] read(FormatReader reader) throws IOException {
            BigInteger length = natCodec.read(reader);
            if(length.compareTo(BigInteger.valueOf(Integer.MAX_VALUE)) > 0) {
                throw new ArithmeticException("Length of array would overflow");
            }

            short[] data = new short[length.intValue()];
            for(int i = 0; i < data.length; ++i) {
                data[i] = reader.readShort();
            }
            return data;
        }

        @Override
        public void write(FormatWriter writer, short[] value) throws IOException {
            natCodec.write(writer, BigInteger.valueOf(value.length));
            for(int i = 0; i < value.length; ++i) {
                writer.writeShort(value[i]);
            }
        }

    };

    public static final Codec<int[]> i32ListCodec = new Codec<int[]>() {

        @Override
        public int[] read(FormatReader reader) throws IOException {
            BigInteger length = natCodec.read(reader);
            if(length.compareTo(BigInteger.valueOf(Integer.MAX_VALUE)) > 0) {
                throw new ArithmeticException("Length of array would overflow");
            }

            int[] data = new int[length.intValue()];
            for(int i = 0; i < data.length; ++i) {
                data[i] = reader.readInt();
            }
            return data;
        }

        @Override
        public void write(FormatWriter writer, int[] value) throws IOException {
            natCodec.write(writer, BigInteger.valueOf(value.length));
            for(int i = 0; i < value.length; ++i) {
                writer.writeInt(value[i]);
            }
        }

    };

    public static final Codec<long[]> i64ListCodec = new Codec<long[]>() {

        @Override
        public long[] read(FormatReader reader) throws IOException {
            BigInteger length = natCodec.read(reader);
            if(length.compareTo(BigInteger.valueOf(Integer.MAX_VALUE)) > 0) {
                throw new ArithmeticException("Length of array would overflow");
            }

            long[] data = new long[length.intValue()];
            for(int i = 0; i < data.length; ++i) {
                data[i] = reader.readLong();
            }
            return data;
        }

        @Override
        public void write(FormatWriter writer, long[] value) throws IOException {
            natCodec.write(writer, BigInteger.valueOf(value.length));
            for(int i = 0; i < value.length; ++i) {
                writer.writeLong(value[i]);
            }
        }

    };

    public static <T> Codec<List<T>> listCodec(Codec<T> elementCodec) {
        return new Codec<List<T>>() {

            @Override
            public List<T> read(FormatReader reader) throws IOException {
                BigInteger length = natCodec.read(reader);
                if(length.compareTo(BigInteger.valueOf(Integer.MAX_VALUE)) > 0) {
                    throw new ArithmeticException("Length of list would overflow");
                }

                int len = length.intValue();
    
                List<T> data = new ArrayList<>(len);
                for(int i = 0; i < data.size(); ++i) {
                    data.add(elementCodec.read(reader));
                }
                return data;
            }
    
            @Override
            public void write(FormatWriter writer, List<T> value) throws IOException {
                natCodec.write(writer, BigInteger.valueOf(value.size()));
                for(int i = 0; i < value.size(); ++i) {
                    elementCodec.write(writer, value.get(i));
                }
            }
    
        };
    }

    public static <T> Codec<Optional<T>> optionalCodec(Codec<T> elementCodec) {
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
