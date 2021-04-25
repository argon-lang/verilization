package struct.versions;

import dev.argon.verilization.java_runtime.Converter;

final class Main_Conversions {
    private Main_Conversions() {}
    
    static final Converter<Main.V3, Main.V4> v3ToV4 = new Converter<Main.V3, Main.V4>() {
        @Override
        public Main.V4 convert(Main.V3 prev) {
            return new Main.V4(
                prev.n,
                prev.m,
                Referenced.V4.fromV3.convert(prev.r),
                new Addition.V4(
                    5
                )
            );
        }
    };

}
