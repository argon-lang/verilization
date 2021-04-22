package struct.versions;

final class Main_Conversions {
    private Main_Conversions() {}
    
    static Main.V4 v3ToV4(Main.V3 prev) {
        return new Main.V4(
            prev.n,
            prev.m,
            Referenced.V4.fromV3.apply(prev.r),
            new Addition.V4(
                5
            )
        );
    }

}
