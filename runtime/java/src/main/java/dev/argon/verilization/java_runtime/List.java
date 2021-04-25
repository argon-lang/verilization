package dev.argon.verilization.java_runtime;

import java.lang.reflect.Array;
import java.math.BigInteger;
import java.io.IOException;

public abstract class List<A> {
    List() {}

    public abstract int size();
    public abstract A get(int index);



    public static <A, B> Converter<List<A>, List<B>> converter(Converter<A, B> elementConverter) {
        if(elementConverter instanceof IdentityConverter<?>) {
            @SuppressWarnings("unchecked")
            var converter = (Converter<List<A>, List<B>>)new IdentityConverter<A>();
            return converter;
        }

        return new Converter<List<A>, List<B>>() {
            @Override
            public List<B> convert(List<A> prev) {
                @SuppressWarnings("unchecked")
                B[] values = (B[])new Object[prev.size()];

                for(int i = 0; i < prev.size(); ++i) {
                    values[i] = elementConverter.convert(prev.get(i));
                }

                return new ObjectList<B>(values);
            }
        };
    }
    

    public static <A> Codec<List<A>> codec(Codec<A> elementCodec) {
        if(elementCodec == StandardCodecs.i8Codec) {
            @SuppressWarnings("unchecked")
            var codec = (Codec<List<A>>)(Object)i8ListCodec;
            return codec;
        }
        else if(elementCodec == StandardCodecs.i16Codec) {
            @SuppressWarnings("unchecked")
            var codec = (Codec<List<A>>)(Object)i16ListCodec;
            return codec;
        }
        else if(elementCodec == StandardCodecs.i32Codec) {
            @SuppressWarnings("unchecked")
            var codec = (Codec<List<A>>)(Object)i32ListCodec;
            return codec;
        }
        else if(elementCodec == StandardCodecs.i64Codec) {
            @SuppressWarnings("unchecked")
            var codec = (Codec<List<A>>)(Object)i64ListCodec;
            return codec;
        }

        return new Codec<List<A>>() {

            @Override
            public List<A> read(FormatReader reader) throws IOException {
                BigInteger length = StandardCodecs.natCodec.read(reader);

                @SuppressWarnings("unchecked")
                A[] values = (A[])new Object[length.intValueExact()];

                for(int i = 0; i < values.length; ++i) {
                    values[i] = elementCodec.read(reader);
                }
                return new ObjectList<A>(values);
            }
    
            @Override
            public void write(FormatWriter writer, List<A> value) throws IOException {
                StandardCodecs.natCodec.write(writer, BigInteger.valueOf(value.size()));
                for(int i = 0; i < value.size(); ++i) {
                    elementCodec.write(writer, value.get(i));
                }
            }
    
        };
    }

    private static final Codec<List<Byte>> i8ListCodec = new Codec<List<Byte>>() {

        @Override
        public List<Byte> read(FormatReader reader) throws IOException {
            BigInteger length = StandardCodecs.natCodec.read(reader);

            return new ByteList(reader.readBytes(length.intValueExact()));
        }

        @Override
        public void write(FormatWriter writer, List<Byte> value) throws IOException {
            var list = ByteList.unbox(value);
            StandardCodecs.natCodec.write(writer, BigInteger.valueOf(list.size()));
            writer.writeBytes(list.values);
        }

    };

    private static final Codec<List<Short>> i16ListCodec = new Codec<List<Short>>() {

        @Override
        public List<Short> read(FormatReader reader) throws IOException {
            BigInteger length = StandardCodecs.natCodec.read(reader);

            short[] data = new short[length.intValueExact()];
            for(int i = 0; i < data.length; ++i) {
                data[i] = reader.readShort();
            }
            return new ShortList(data);
        }

        @Override
        public void write(FormatWriter writer, List<Short> value) throws IOException {
            var list = ShortList.unbox(value);
            StandardCodecs.natCodec.write(writer, BigInteger.valueOf(list.size()));
            for(int i = 0; i < list.size(); ++i) {
                writer.writeShort(list.getUnboxed(i));
            }
        }

    };

    private static final Codec<List<Integer>> i32ListCodec = new Codec<List<Integer>>() {

        @Override
        public List<Integer> read(FormatReader reader) throws IOException {
            BigInteger length = StandardCodecs.natCodec.read(reader);

            int[] data = new int[length.intValueExact()];
            for(int i = 0; i < data.length; ++i) {
                data[i] = reader.readInt();
            }
            return new IntList(data);
        }

        @Override
        public void write(FormatWriter writer, List<Integer> value) throws IOException {
            var list = IntList.unbox(value);
            StandardCodecs.natCodec.write(writer, BigInteger.valueOf(list.size()));
            for(int i = 0; i < list.size(); ++i) {
                writer.writeInt(list.getUnboxed(i));
            }
        }

    };

    private static final Codec<List<Long>> i64ListCodec = new Codec<List<Long>>() {

        @Override
        public List<Long> read(FormatReader reader) throws IOException {
            BigInteger length = StandardCodecs.natCodec.read(reader);

            long[] data = new long[length.intValueExact()];
            for(int i = 0; i < data.length; ++i) {
                data[i] = reader.readLong();
            }
            return new LongList(data);
        }

        @Override
        public void write(FormatWriter writer, List<Long> value) throws IOException {
            var list = LongList.unbox(value);
            StandardCodecs.natCodec.write(writer, BigInteger.valueOf(list.size()));
            for(int i = 0; i < list.size(); ++i) {
                writer.writeLong(list.getUnboxed(i));
            }
        }

    };
}
