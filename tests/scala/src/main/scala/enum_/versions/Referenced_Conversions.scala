package enum_.versions

object Referenced_Conversions {
	def v1ToV2(prev: Referenced.V1): Referenced.V2 =
		prev match {
			case Referenced.V1.x(x) => Referenced.V2.x(x)
		}
}
