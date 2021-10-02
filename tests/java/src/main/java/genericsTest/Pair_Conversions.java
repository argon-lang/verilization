package genericsTest;

import dev.argon.verilization.runtime.Converter;

public final class Pair_Conversions {
    private Pair_Conversions() {}

    static <A_1, A_2, B_1, B_2> Converter<Pair.V3<A_1, B_1>, Pair.V4<A_2, B_2>> v3ToV4(Converter<A_1, A_2> A_conv, Converter<B_1, B_2> B_conv) {
        return new Converter<Pair.V3<A_1, B_1>, Pair.V4<A_2, B_2>>() {
            @Override
            public Pair.V4<A_2, B_2> convert(Pair.V3<A_1, B_1> prev) {
                return new Pair.V4<A_2, B_2>(A_conv.convert(prev.left()), B_conv.convert(prev.right()), "dummy");
            }
        };
    }
}
