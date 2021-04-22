package genericsTest;

import java.util.function.Function;

public final class Pair_Conversions {
    private Pair_Conversions() {}

    static <A_1, A_2, B_1, B_2> Pair.V4<A_2, B_2> v3ToV4(Function<A_1, A_2> A_conv, Function<B_1, B_2> B_conv, Pair.V3<A_1, B_1> prev) {
        return new Pair.V4<A_2, B_2>(A_conv.apply(prev.left), B_conv.apply(prev.right), "dummy");
    }
}
