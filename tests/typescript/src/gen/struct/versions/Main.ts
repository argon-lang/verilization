import {Codec, FormatWriter, FormatReader, StandardCodecs} from "@verilization/runtime";
import * as sym_struct_versions_Addition from "./Addition.js";
import * as sym_struct_versions_Referenced from "./Referenced.js";
export interface V1 {
	readonly n: number;
	readonly m: bigint;
	readonly r: sym_struct_versions_Referenced.V1;
}
export namespace V1 {
	export const codec: Codec<V1> = {
		async read(reader: FormatReader): Promise<V1> {
			return {
				n: await StandardCodecs.i32.read(reader),
				m: await StandardCodecs.i64.read(reader),
				r: await sym_struct_versions_Referenced.V1.codec.read(reader),
			};
		},
		async write(writer: FormatWriter, value: V1): Promise<void> {
			await StandardCodecs.i32.write(writer, value.n);
			await StandardCodecs.i64.write(writer, value.m);
			await sym_struct_versions_Referenced.V1.codec.write(writer, value.r);
		},
	};
}
export interface V2 {
	readonly n: number;
	readonly m: bigint;
	readonly r: sym_struct_versions_Referenced.V2;
}
export namespace V2 {
	export function from_v1(prev: V1): V2 {
		return {
			n: prev.n,
			m: prev.m,
			r: sym_struct_versions_Referenced.V2.from_v1(prev.r),
		};
	}
	export const codec: Codec<V2> = {
		async read(reader: FormatReader): Promise<V2> {
			return {
				n: await StandardCodecs.i32.read(reader),
				m: await StandardCodecs.i64.read(reader),
				r: await sym_struct_versions_Referenced.V2.codec.read(reader),
			};
		},
		async write(writer: FormatWriter, value: V2): Promise<void> {
			await StandardCodecs.i32.write(writer, value.n);
			await StandardCodecs.i64.write(writer, value.m);
			await sym_struct_versions_Referenced.V2.codec.write(writer, value.r);
		},
	};
}
export interface V3 {
	readonly n: number;
	readonly m: bigint;
	readonly r: sym_struct_versions_Referenced.V3;
}
export namespace V3 {
	export function from_v2(prev: V2): V3 {
		return {
			n: prev.n,
			m: prev.m,
			r: sym_struct_versions_Referenced.V3.from_v2(prev.r),
		};
	}
	export const codec: Codec<V3> = {
		async read(reader: FormatReader): Promise<V3> {
			return {
				n: await StandardCodecs.i32.read(reader),
				m: await StandardCodecs.i64.read(reader),
				r: await sym_struct_versions_Referenced.V3.codec.read(reader),
			};
		},
		async write(writer: FormatWriter, value: V3): Promise<void> {
			await StandardCodecs.i32.write(writer, value.n);
			await StandardCodecs.i64.write(writer, value.m);
			await sym_struct_versions_Referenced.V3.codec.write(writer, value.r);
		},
	};
}
export interface V4 {
	readonly n: number;
	readonly m: bigint;
	readonly r: sym_struct_versions_Referenced.V4;
	readonly addition: sym_struct_versions_Addition.V4;
}
import {v3_to_v4} from "./Main.conv.js";
export namespace V4 {
	export const from_v3: (prev: V3) => V4 = v3_to_v4;
	export const codec: Codec<V4> = {
		async read(reader: FormatReader): Promise<V4> {
			return {
				n: await StandardCodecs.i32.read(reader),
				m: await StandardCodecs.i64.read(reader),
				r: await sym_struct_versions_Referenced.V4.codec.read(reader),
				addition: await sym_struct_versions_Addition.V4.codec.read(reader),
			};
		},
		async write(writer: FormatWriter, value: V4): Promise<void> {
			await StandardCodecs.i32.write(writer, value.n);
			await StandardCodecs.i64.write(writer, value.m);
			await sym_struct_versions_Referenced.V4.codec.write(writer, value.r);
			await sym_struct_versions_Addition.V4.codec.write(writer, value.addition);
		},
	};
}
