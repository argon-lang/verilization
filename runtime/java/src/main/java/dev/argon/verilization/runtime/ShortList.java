package dev.argon.verilization.runtime;

import java.util.Arrays;

public final class ShortList extends List<Short> {
    ShortList(short[] values) {
        this.values = values;
    }

    private final short[] values;

    @Override
    public int size() {
        return values.length;
    }

    @Override
    public Short get(int index) {
        return values[index];
    }

    public short getUnboxed(int index) {
        return values[index];
    }

    public static ShortList unbox(List<Short> l) {
        if(l instanceof ShortList l2) {
            return l2;
        }

        short[] values = new short[l.size()];
        for(int i = 0; i < values.length; ++i) {
            values[i] = l.get(i);
        }

        return new ShortList(values);
    }

    @Override
    public int hashCode() {
        return Arrays.hashCode(values);
    }

    @Override
    public boolean equals(Object obj) {
        if(!(obj instanceof ShortList other)) {
            return false;
        }

        return Arrays.equals(values, other.values);
    }
}
