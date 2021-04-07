import {Codec, FormatWriter, FormatReader, StandardCodecs} from "@verilization/runtime";
import * as sym_enum_versions_Referenced from "./Referenced.js";
import * as sym_enum_versions_Addition from "./Addition.js";
export type V1 = { readonly tag: "n", readonly n: number, }
	| { readonly tag: "m", readonly m: bigint, }
	| { readonly tag: "r", readonly r: sym_enum_versions_Referenced.V1, };
export namespace V1 {
	export const codec: Codec<V1> = {
		async read(reader: FormatReader): Promise<V1> {
			const tag = await StandardCodecs.nat.read(reader);
			switch(tag) {
				case 0n: return { tag: "n", "n": await StandardCodecs.i32.read(reader) };
				case 1n: return { tag: "m", "m": await StandardCodecs.i64.read(reader) };
				case 2n: return { tag: "r", "r": await sym_enum_versions_Referenced.V1.codec.read(reader) };
				default: throw new Error("Unknown tag");
			};
		},
		async write(writer: FormatWriter, value: V1): Promise<void> {
			switch(value.tag) {
				case "n":
					await StandardCodecs.nat.write(writer, 0n);
					await StandardCodecs.i32.write(writer, value.n);
					break;
				case "m":
					await StandardCodecs.nat.write(writer, 1n);
					await StandardCodecs.i64.write(writer, value.m);
					break;
				case "r":
					await StandardCodecs.nat.write(writer, 2n);
					await sym_enum_versions_Referenced.V1.codec.write(writer, value.r);
					break;
				default: throw new Error("Unknown tag");
			}
		},
	};
}
export type V2 = { readonly tag: "n", readonly n: number, }
	| { readonly tag: "m", readonly m: bigint, }
	| { readonly tag: "r", readonly r: sym_enum_versions_Referenced.V2, };
export namespace V2 {
	export function from_v1(prev: V1): V2 {
		switch(prev.tag) {
			case "n": return { tag: "n", "n": prev.n};
			case "m": return { tag: "m", "m": prev.m};
			case "r": return { tag: "r", "r": sym_enum_versions_Referenced.V2.from_v1(prev.r)};
			default: return prev;
		}
	}
	export const codec: Codec<V2> = {
		async read(reader: FormatReader): Promise<V2> {
			const tag = await StandardCodecs.nat.read(reader);
			switch(tag) {
				case 0n: return { tag: "n", "n": await StandardCodecs.i32.read(reader) };
				case 1n: return { tag: "m", "m": await StandardCodecs.i64.read(reader) };
				case 2n: return { tag: "r", "r": await sym_enum_versions_Referenced.V2.codec.read(reader) };
				default: throw new Error("Unknown tag");
			};
		},
		async write(writer: FormatWriter, value: V2): Promise<void> {
			switch(value.tag) {
				case "n":
					await StandardCodecs.nat.write(writer, 0n);
					await StandardCodecs.i32.write(writer, value.n);
					break;
				case "m":
					await StandardCodecs.nat.write(writer, 1n);
					await StandardCodecs.i64.write(writer, value.m);
					break;
				case "r":
					await StandardCodecs.nat.write(writer, 2n);
					await sym_enum_versions_Referenced.V2.codec.write(writer, value.r);
					break;
				default: throw new Error("Unknown tag");
			}
		},
	};
}
export type V3 = { readonly tag: "n", readonly n: number, }
	| { readonly tag: "m", readonly m: bigint, }
	| { readonly tag: "r", readonly r: sym_enum_versions_Referenced.V3, };
export namespace V3 {
	export function from_v2(prev: V2): V3 {
		switch(prev.tag) {
			case "n": return { tag: "n", "n": prev.n};
			case "m": return { tag: "m", "m": prev.m};
			case "r": return { tag: "r", "r": sym_enum_versions_Referenced.V3.from_v2(prev.r)};
			default: return prev;
		}
	}
	export const codec: Codec<V3> = {
		async read(reader: FormatReader): Promise<V3> {
			const tag = await StandardCodecs.nat.read(reader);
			switch(tag) {
				case 0n: return { tag: "n", "n": await StandardCodecs.i32.read(reader) };
				case 1n: return { tag: "m", "m": await StandardCodecs.i64.read(reader) };
				case 2n: return { tag: "r", "r": await sym_enum_versions_Referenced.V3.codec.read(reader) };
				default: throw new Error("Unknown tag");
			};
		},
		async write(writer: FormatWriter, value: V3): Promise<void> {
			switch(value.tag) {
				case "n":
					await StandardCodecs.nat.write(writer, 0n);
					await StandardCodecs.i32.write(writer, value.n);
					break;
				case "m":
					await StandardCodecs.nat.write(writer, 1n);
					await StandardCodecs.i64.write(writer, value.m);
					break;
				case "r":
					await StandardCodecs.nat.write(writer, 2n);
					await sym_enum_versions_Referenced.V3.codec.write(writer, value.r);
					break;
				default: throw new Error("Unknown tag");
			}
		},
	};
}
export type V4 = { readonly tag: "n", readonly n: number, }
	| { readonly tag: "m", readonly m: bigint, }
	| { readonly tag: "r", readonly r: sym_enum_versions_Referenced.V4, }
	| { readonly tag: "addition", readonly addition: sym_enum_versions_Addition.V4, };
import {v3_to_v4} from "./Main.conv.js";
export namespace V4 {
	export const from_v3: (prev: V3) => V4 = v3_to_v4;
	export const codec: Codec<V4> = {
		async read(reader: FormatReader): Promise<V4> {
			const tag = await StandardCodecs.nat.read(reader);
			switch(tag) {
				case 0n: return { tag: "n", "n": await StandardCodecs.i32.read(reader) };
				case 1n: return { tag: "m", "m": await StandardCodecs.i64.read(reader) };
				case 2n: return { tag: "r", "r": await sym_enum_versions_Referenced.V4.codec.read(reader) };
				case 3n: return { tag: "addition", "addition": await sym_enum_versions_Addition.V4.codec.read(reader) };
				default: throw new Error("Unknown tag");
			};
		},
		async write(writer: FormatWriter, value: V4): Promise<void> {
			switch(value.tag) {
				case "n":
					await StandardCodecs.nat.write(writer, 0n);
					await StandardCodecs.i32.write(writer, value.n);
					break;
				case "m":
					await StandardCodecs.nat.write(writer, 1n);
					await StandardCodecs.i64.write(writer, value.m);
					break;
				case "r":
					await StandardCodecs.nat.write(writer, 2n);
					await sym_enum_versions_Referenced.V4.codec.write(writer, value.r);
					break;
				case "addition":
					await StandardCodecs.nat.write(writer, 3n);
					await sym_enum_versions_Addition.V4.codec.write(writer, value.addition);
					break;
				default: throw new Error("Unknown tag");
			}
		},
	};
}
