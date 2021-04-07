import {Codec, FormatWriter, FormatReader, StandardCodecs} from "@verilization/runtime";
export interface V1 {
	readonly x: number;
}
export namespace V1 {
	export const codec: Codec<V1> = {
		async read(reader: FormatReader): Promise<V1> {
			return {
				x: await StandardCodecs.i32.read(reader),
			};
		},
		async write(writer: FormatWriter, value: V1): Promise<void> {
			await StandardCodecs.i32.write(writer, value.x);
		},
	};
}
export interface V2 {
	readonly x: bigint;
}
import {v1_to_v2} from "./Referenced.conv.js";
export namespace V2 {
	export const from_v1: (prev: V1) => V2 = v1_to_v2;
	export const codec: Codec<V2> = {
		async read(reader: FormatReader): Promise<V2> {
			return {
				x: await StandardCodecs.i64.read(reader),
			};
		},
		async write(writer: FormatWriter, value: V2): Promise<void> {
			await StandardCodecs.i64.write(writer, value.x);
		},
	};
}
export interface V3 {
	readonly x: bigint;
}
export namespace V3 {
	export function from_v2(prev: V2): V3 {
		return {
			x: prev.x,
		};
	}
	export const codec: Codec<V3> = {
		async read(reader: FormatReader): Promise<V3> {
			return {
				x: await StandardCodecs.i64.read(reader),
			};
		},
		async write(writer: FormatWriter, value: V3): Promise<void> {
			await StandardCodecs.i64.write(writer, value.x);
		},
	};
}
export interface V4 {
	readonly x: bigint;
}
export namespace V4 {
	export function from_v3(prev: V3): V4 {
		return {
			x: prev.x,
		};
	}
	export const codec: Codec<V4> = {
		async read(reader: FormatReader): Promise<V4> {
			return {
				x: await StandardCodecs.i64.read(reader),
			};
		},
		async write(writer: FormatWriter, value: V4): Promise<void> {
			await StandardCodecs.i64.write(writer, value.x);
		},
	};
}
