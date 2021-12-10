package genericsTest

import dev.argon.verilization.scala_runtime.Converter

private[genericsTest] object Pair_Conversions {
    def v3ToV4[A_1, A_2, B_1, B_2](A_conv: Converter[A_1, A_2], B_conv: Converter[B_1, B_2]): Converter[Pair.V3[A_1, B_1], Pair.V4[A_2, B_2]] = prev =>
        Pair.V4(A_conv.convert(prev.left), B_conv.convert(prev.right), "dummy")
}
