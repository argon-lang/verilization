package genericsTest

import dev.argon.verilization.scala_runtime.Converter

private[genericsTest] object Either_Conversions {
    def v3ToV4[A_1, A_2, B_1, B_2](A_conv: Converter[A_1, A_2], B_conv: Converter[B_1, B_2]): Converter[Either.V3[A_1, B_1], Either.V4[A_2, B_2]] = {
        case prev: Either.V3.left[A_1, B_1] => Either.V4.left(A_conv.convert(prev.left))
        case prev: Either.V3.right[A_1, B_1] => Either.V4.right(B_conv.convert(prev.right))
    }
}
