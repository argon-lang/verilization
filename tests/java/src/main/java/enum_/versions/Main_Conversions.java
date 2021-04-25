package enum_.versions;

import dev.argon.verilization.java_runtime.Converter;

final class Main_Conversions {
    private Main_Conversions() {}
    
    static final Converter<Main.V3, Main.V4> v3ToV4 = new Converter<Main.V3, Main.V4>() {
        @Override
        public Main.V4 convert(Main.V3 prev) {
            if(prev instanceof Main.V3.n) {
                return new Main.V4.n(((Main.V3.n)prev).n);
            }
            else if(prev instanceof Main.V3.m) {
                return new Main.V4.m(((Main.V3.m)prev).m);
            }
            else if(prev instanceof Main.V3.r) {
                return new Main.V4.r(Referenced.V4.fromV3.convert(((Main.V3.r)prev).r));
            }
            else {
                throw new IllegalArgumentException();
            }
        }
    };
    

}
