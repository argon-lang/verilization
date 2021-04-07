import {V3, V4} from "./Main.js";
import * as Referenced from "./Referenced.js";

export function v3_to_v4(v3: V3): V4 {
    return {
        n: v3.n,
        m: v3.m,
        r: Referenced.V4.from_v3(v3.r),
        addition: {
            stuff: 5,
        },
    };
}
