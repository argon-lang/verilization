package struct.versions;

final class Referenced_Conversions {
    private Referenced_Conversions() {}


    static Referenced.V2 v1ToV2(Referenced.V1 prev) {
        return new Referenced.V2(prev.x);
    }

}
