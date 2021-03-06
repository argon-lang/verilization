package dev.argon.verilization.runtime;

import java.util.Arrays;

public final class ByteList extends List<Byte> {
    ByteList(byte[] values) {
        this.values = values;
    }

    final byte[] values;

    @Override
    public int size() {
        return values.length;
    }

    @Override
    public Byte get(int index) {
        return values[index];
    }

    public byte getUnboxed(int index) {
        return values[index];
    }

    public static ByteList unbox(List<Byte> l) {
        if(l instanceof ByteList l2) {
            return l2;
        }

        byte[] values = new byte[l.size()];
        for(int i = 0; i < values.length; ++i) {
            values[i] = l.get(i);
        }

        return new ByteList(values);
    }

    @Override
    public int hashCode() {
        return Arrays.hashCode(values);
    }

    @Override
    public boolean equals(Object obj) {
        if(!(obj instanceof ByteList other)) {
            return false;
        }

        return Arrays.equals(values, other.values);
    }
}
