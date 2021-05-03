package dev.argon.verilization.runtime;

import java.util.Arrays;

public final class LongList extends List<Long> {
    LongList(long[] values) {
        this.values = values;
    }

    private final long[] values;

    @Override
    public int size() {
        return values.length;
    }

    @Override
    public Long get(int index) {
        return values[index];
    }

    public long getUnboxed(int index) {
        return values[index];
    }

    public static LongList unbox(List<Long> l) {
        if(l instanceof LongList) {
            return (LongList)l;
        }

        long[] values = new long[l.size()];
        for(int i = 0; i < values.length; ++i) {
            values[i] = l.get(i);
        }

        return new LongList(values);
    }

    @Override
    public int hashCode() {
        return Arrays.hashCode(values);
    }

    @Override
    public boolean equals(Object obj) {
        if(!(obj instanceof LongList)) {
            return false;
        }

        var other = (LongList)obj;
        return Arrays.equals(values, other.values);
    }
}
