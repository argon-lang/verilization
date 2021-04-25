package dev.argon.verilization.java_runtime;

import java.util.Arrays;

final class ObjectList<A> extends List<A> {
    ObjectList(A[] values) {
        this.values = values;
    }

    private final A[] values;

    @Override
    public int size() {
        return values.length;
    }

    @Override
    public A get(int index) {
        return values[index];
    }

    @Override
    public int hashCode() {
        return Arrays.hashCode(values);
    }

    @Override
    public boolean equals(Object obj) {
        if(!(obj instanceof ObjectList<?>)) {
            return false;
        }

        var other = (ObjectList<?>)obj;
        return Arrays.equals(values, other.values);
    }
}
