import {Codec, FormatWriter, FormatReader, StandardCodecs} from "@verilization/runtime";
export type V4 = { readonly tag: "stuff", readonly stuff: number, };
export namespace V4 {
	export const codec: Codec<V4> = {
		async read(reader: FormatReader): Promise<V4> {
			const tag = await StandardCodecs.nat.read(reader);
			switch(tag) {
				case 0n: return { tag: "stuff", "stuff": await StandardCodecs.i32.read(reader) };
				default: throw new Error("Unknown tag");
			};
		},
		async write(writer: FormatWriter, value: V4): Promise<void> {
			switch(value.tag) {
				case "stuff":
					await StandardCodecs.nat.write(writer, 0n);
					await StandardCodecs.i32.write(writer, value.stuff);
					break;
				default: throw new Error("Unknown tag");
			}
		},
	};
}
