package enum_.versions

import dev.argon.verilization.scala_runtime.Converter

object Referenced_Conversions {
	val v1ToV2: Converter[Referenced.V1, Referenced.V2] = {
		case Referenced.V1.X(x) => Referenced.V2.X(x)
	}
}
