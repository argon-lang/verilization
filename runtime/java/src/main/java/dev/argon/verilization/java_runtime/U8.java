package dev.argon.verilization.java_runtime;

public final class U8 {
    private U8() {}

    public static byte fromInteger(int i) {
        return (byte)i;
    }

    public static final Codec<Byte> codec = I8.codec;
}
