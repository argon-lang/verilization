
import {V3, V4} from "./Pair.js";

export function v3_to_v4<A_1, A_2, B_1, B_2>(a_conv: (prev: A_1) => A_2, b_conv: (prev: B_1) => B_2, prev: V3<A_1, B_1>): V4<A_2, B_2> {
    return {
        left: a_conv(prev.left),
        right: b_conv(prev.right),
        other: "dummy",
    };
}
