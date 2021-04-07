package struct.versions

object Referenced_Conversions {
    def v1ToV2(prev: Referenced.V1): Referenced.V2 =
        Referenced.V2(x = prev.x)
}
