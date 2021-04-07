import {Codec, FormatWriter, FormatReader, StandardCodecs} from "@verilization/runtime";
export type V1 = { readonly tag: "x", readonly x: number, };
export namespace V1 {
	export const codec: Codec<V1> = {
		async read(reader: FormatReader): Promise<V1> {
			const tag = await StandardCodecs.nat.read(reader);
			switch(tag) {
				case 0n: return { tag: "x", "x": await StandardCodecs.i32.read(reader) };
				default: throw new Error("Unknown tag");
			};
		},
		async write(writer: FormatWriter, value: V1): Promise<void> {
			switch(value.tag) {
				case "x":
					await StandardCodecs.nat.write(writer, 0n);
					await StandardCodecs.i32.write(writer, value.x);
					break;
				default: throw new Error("Unknown tag");
			}
		},
	};
}
export type V2 = { readonly tag: "x", readonly x: bigint, };
import {v1_to_v2} from "./Referenced.conv.js";
export namespace V2 {
	export const from_v1: (prev: V1) => V2 = v1_to_v2;
	export const codec: Codec<V2> = {
		async read(reader: FormatReader): Promise<V2> {
			const tag = await StandardCodecs.nat.read(reader);
			switch(tag) {
				case 0n: return { tag: "x", "x": await StandardCodecs.i64.read(reader) };
				default: throw new Error("Unknown tag");
			};
		},
		async write(writer: FormatWriter, value: V2): Promise<void> {
			switch(value.tag) {
				case "x":
					await StandardCodecs.nat.write(writer, 0n);
					await StandardCodecs.i64.write(writer, value.x);
					break;
				default: throw new Error("Unknown tag");
			}
		},
	};
}
export type V3 = { readonly tag: "x", readonly x: bigint, };
export namespace V3 {
	export function from_v2(prev: V2): V3 {
		switch(prev.tag) {
			case "x": return { tag: "x", "x": prev.x};
			default: return prev;
		}
	}
	export const codec: Codec<V3> = {
		async read(reader: FormatReader): Promise<V3> {
			const tag = await StandardCodecs.nat.read(reader);
			switch(tag) {
				case 0n: return { tag: "x", "x": await StandardCodecs.i64.read(reader) };
				default: throw new Error("Unknown tag");
			};
		},
		async write(writer: FormatWriter, value: V3): Promise<void> {
			switch(value.tag) {
				case "x":
					await StandardCodecs.nat.write(writer, 0n);
					await StandardCodecs.i64.write(writer, value.x);
					break;
				default: throw new Error("Unknown tag");
			}
		},
	};
}
export type V4 = { readonly tag: "x", readonly x: bigint, };
export namespace V4 {
	export function from_v3(prev: V3): V4 {
		switch(prev.tag) {
			case "x": return { tag: "x", "x": prev.x};
			default: return prev;
		}
	}
	export const codec: Codec<V4> = {
		async read(reader: FormatReader): Promise<V4> {
			const tag = await StandardCodecs.nat.read(reader);
			switch(tag) {
				case 0n: return { tag: "x", "x": await StandardCodecs.i64.read(reader) };
				default: throw new Error("Unknown tag");
			};
		},
		async write(writer: FormatWriter, value: V4): Promise<void> {
			switch(value.tag) {
				case "x":
					await StandardCodecs.nat.write(writer, 0n);
					await StandardCodecs.i64.write(writer, value.x);
					break;
				default: throw new Error("Unknown tag");
			}
		},
	};
}
