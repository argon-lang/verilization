package enum_.versions

import dev.argon.verilization.scala_runtime.Converter

object Main_Conversions {
	val v3ToV4: Converter[Main.V3, Main.V4] = {
		case Main.V3.N(n) => Main.V4.N(n)
		case Main.V3.M(m) => Main.V4.M(m)
		case Main.V3.R(r) => Main.V4.R(Referenced.V4.fromV3.convert(r))
	}
}
