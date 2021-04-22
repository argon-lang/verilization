
import {V3, V4} from "./Either.js";

export function v3_to_v4<A_1, A_2, B_1, B_2>(a_conv: (prev: A_1) => A_2, b_conv: (prev: B_1) => B_2, prev: V3<A_1, B_1>): V4<A_2, B_2> {
    switch(prev.tag) {
        case "left": return { tag: "left", left: a_conv(prev.left), };
        case "right": return { tag: "right", right: b_conv(prev.right), };
        default: return prev;
    }
}
