package dev.argon.verilization.java_runtime;

import java.util.Arrays;

public final class IntList extends List<Integer> {
    IntList(int[] values) {
        this.values = values;
    }

    private final int[] values;

    @Override
    public int size() {
        return values.length;
    }

    @Override
    public Integer get(int index) {
        return values[index];
    }

    public int getUnboxed(int index) {
        return values[index];
    }

    public static IntList unbox(List<Integer> l) {
        if(l instanceof IntList) {
            return (IntList)l;
        }

        int[] values = new int[l.size()];
        for(int i = 0; i < values.length; ++i) {
            values[i] = l.get(i);
        }

        return new IntList(values);
    }

    @Override
    public int hashCode() {
        return Arrays.hashCode(values);
    }

    @Override
    public boolean equals(Object obj) {
        if(!(obj instanceof IntList)) {
            return false;
        }

        var other = (IntList)obj;
        return Arrays.equals(values, other.values);
    }
}
