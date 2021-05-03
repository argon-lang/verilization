package dev.argon.verilization.runtime;


public interface Converter<A, B> {
    B convert(A value);

    public static <A> Converter<A, A> identity() {
        return new IdentityConverter<A>();
    }
}
