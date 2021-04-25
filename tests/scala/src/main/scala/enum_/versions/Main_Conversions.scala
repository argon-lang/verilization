package enum_.versions

import dev.argon.verilization.scala_runtime.Converter

object Main_Conversions {
	val v3ToV4: Converter[Main.V3, Main.V4] = {
		case Main.V3.n(n) => Main.V4.n(n)
		case Main.V3.m(m) => Main.V4.m(m)
		case Main.V3.r(r) => Main.V4.r(Referenced.V4.fromV3.convert(r))
	}
}
