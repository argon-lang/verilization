package enum_.versions;

public abstract class Referenced_Conversions {
    private Referenced_Conversions() {}


    public static Referenced.V2 v1ToV2(Referenced.V1 prev) {
        return new Referenced.V2.x(((Referenced.V1.x)prev).x);
    }

}
