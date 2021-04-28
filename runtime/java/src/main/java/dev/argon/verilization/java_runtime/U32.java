package dev.argon.verilization.java_runtime;

public final class U32 {
    private U32() {}

    public static int fromInteger(int i) {
        return i;
    }

    public static int fromInteger(long l) {
        return (int)l;
    }

    public static final Codec<Integer> codec = I32.codec;
}
