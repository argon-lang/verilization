package enum_.versions

object Main_Conversions {
	def v3ToV4(prev: Main.V3): Main.V4 =
		prev match {
			case Main.V3.n(n) => Main.V4.n(n)
			case Main.V3.m(m) => Main.V4.m(m)
			case Main.V3.r(r) => Main.V4.r(Referenced.V4.fromV3(r))
		}
}
