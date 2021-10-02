package genericsTest;

import dev.argon.verilization.runtime.Converter;

final class Either_Conversions {
    private Either_Conversions() {}

    static <A_1, A_2, B_1, B_2> Converter<Either.V3<A_1, B_1>, Either.V4<A_2, B_2>> v3ToV4(Converter<A_1, A_2> A_conv, Converter<B_1, B_2> B_conv) {
        return new Converter<Either.V3<A_1, B_1>, Either.V4<A_2, B_2>>() {
            @Override
            public Either.V4<A_2, B_2> convert(Either.V3<A_1, B_1> prev) {
                return switch(prev) {
                    case Either.V3.Left<A_1, B_1> prev2 -> new Either.V4.Left<A_2, B_2>(A_conv.convert(prev2.left()));
                    case Either.V3.Right<A_1, B_1> prev2 -> new Either.V4.Right<A_2, B_2>(B_conv.convert(prev2.right()));
                };
            }
        };
    }
}
