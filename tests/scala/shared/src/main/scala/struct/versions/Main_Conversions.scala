package struct.versions

import dev.argon.verilization.scala_runtime.Converter

object Main_Conversions {
    val v3ToV4: Converter[Main.V3, Main.V4] = prev =>
        Main.V4(
            n = prev.n,
            m = prev.m,
            r = Referenced.V4.fromV3.convert(prev.r),
            addition = Addition.V4(5),
        )
}