import {V3, V4} from "./Main.js";
import * as Referenced from "./Referenced.js";
import {Converter} from "@verilization/runtime";

export const v3_to_v4: Converter<V3, V4> = {
    convert(v3: V3): V4 {
        return {
            n: v3.n,
            m: v3.m,
            r: Referenced.V4.fromV3.convert(v3.r),
            addition: {
                stuff: 5,
            },
        };
    },
};
