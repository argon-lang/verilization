package struct.versions

import dev.argon.verilization.scala_runtime.Converter

object Referenced_Conversions {
    val v1ToV2: Converter[Referenced.V1, Referenced.V2] = prev =>
        Referenced.V2(x = prev.x)
}
