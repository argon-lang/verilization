package genericsTest;

import java.util.function.Function;

final class Either_Conversions {
    private Either_Conversions() {}

    static <A_1, A_2, B_1, B_2> Either.V4<A_2, B_2> v3ToV4(Function<A_1, A_2> A_conv, Function<B_1, B_2> B_conv, Either.V3<A_1, B_1> prev) {
        if(prev instanceof Either.V3.left<?, ?>) {
            return new Either.V4.left<A_2, B_2>(A_conv.apply(((Either.V3.left<A_1, B_1>)prev).left));
        }
        else if(prev instanceof Either.V3.right<?, ?>) {
            return new Either.V4.right<A_2, B_2>(B_conv.apply(((Either.V3.right<A_1, B_1>)prev).right));
        }
        else {
            throw new IllegalArgumentException();
        }
    }
}
