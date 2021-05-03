package dev.argon.verilization.runtime;

import java.math.BigInteger;

public final class U64 {
    private U64() {}

    public static long fromInteger(int i) {
        return i;
    }

    public static long fromInteger(long l) {
        return l;
    }

    public static long fromInteger(BigInteger i) {
        return i.longValue();
    }

    public static final Codec<Long> codec = I64.codec;
}
