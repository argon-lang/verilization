package struct.versions;

public abstract class Main_Conversions {
    private Main_Conversions() {}
    
    public static Main.V4 v3ToV4(Main.V3 prev) {
        return new Main.V4(
            prev.n,
            prev.m,
            Referenced.V4.fromV3(prev.r),
            new Addition.V4(
                5
            )
        );
    }

}
