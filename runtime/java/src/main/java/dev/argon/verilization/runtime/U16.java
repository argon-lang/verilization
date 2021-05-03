package dev.argon.verilization.runtime;

public final class U16 {
    private U16() {}

    public static short fromInteger(int i) {
        return (short)i;
    }

    public static final Codec<Short> codec = I16.codec;
}
