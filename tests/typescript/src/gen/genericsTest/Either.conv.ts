
import {V3, V4} from "./Either.js";
import {Converter} from "@verilization/runtime";

export function v3_to_v4<A_1, A_2, B_1, B_2>(a_conv: Converter<A_1, A_2>, b_conv: Converter<B_1, B_2>): Converter<V3<A_1, B_1>, V4<A_2, B_2>> {
    return {
        convert(prev: V3<A_1, B_1>): V4<A_2, B_2> {
            switch(prev.tag) {
                case "left": return { tag: "left", left: a_conv.convert(prev.left), };
                case "right": return { tag: "right", right: b_conv.convert(prev.right), };
                default: return prev;
            }
        },
    };
}
