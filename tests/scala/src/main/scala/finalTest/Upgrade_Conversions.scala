package finalTest

object Upgrade_Conversions {
    def v3ToV4(prev: Upgrade.V3): Upgrade.V4 = Upgrade.V4(prev.n)
}
