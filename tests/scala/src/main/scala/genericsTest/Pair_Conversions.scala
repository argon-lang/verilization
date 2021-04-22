package genericsTest

private[genericsTest] object Pair_Conversions {
    def v3ToV4[A_1, A_2, B_1, B_2](A_conv: A_1 => A_2, B_conv: B_1 => B_2)(prev: Pair.V3[A_1, B_1]): Pair.V4[A_2, B_2] =
        Pair.V4(A_conv(prev.left), B_conv(prev.right), "dummy")
}
