package struct.versions

object Main_Conversions {
    def v3ToV4(prev: Main.V3): Main.V4 =
        Main.V4(
            n = prev.n,
            m = prev.m,
            r = Referenced.V4.fromV3(prev.r),
            addition = Addition.V4(5),
        )
}