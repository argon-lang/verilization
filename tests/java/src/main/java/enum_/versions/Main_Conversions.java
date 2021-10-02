package enum_.versions;

import dev.argon.verilization.runtime.Converter;

final class Main_Conversions {
    private Main_Conversions() {}
    
    static final Converter<Main.V3, Main.V4> v3ToV4 = new Converter<Main.V3, Main.V4>() {
        @Override
        public Main.V4 convert(Main.V3 prev) {
            return switch(prev) {
                case Main.V3.N prev2 -> new Main.V4.N(prev2.n());
                case Main.V3.M prev2 -> new Main.V4.M(prev2.m());
                case Main.V3.R prev2 -> new Main.V4.R(Referenced.V4.fromV3.convert(prev2.r()));
            };
        }
    };
    

}
