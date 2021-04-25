package dev.argon.verilization.java_runtime;

public final class IdentityConverter<A> implements Converter<A, A> {
    @Override
    public A convert(A value) {
        return value;
    }
}
