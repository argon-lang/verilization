import {V3, V4} from "./Main.js";
import * as Referenced from "./Referenced.js";
import {Converter} from "@verilization/runtime";

export const v3_to_v4: Converter<V3, V4> = {
    convert(v3: V3): V4 {
        switch(v3.tag) {
            case "n": return { tag: "n", n: v3.n };
            case "m": return { tag: "m", m: v3.m };
            case "r": return { tag: "r", r: Referenced.V4.fromV3.convert(v3.r) };
            default: return v3;
        }
    },
};
