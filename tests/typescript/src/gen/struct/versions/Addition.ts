import {Codec, FormatWriter, FormatReader, StandardCodecs} from "@verilization/runtime";
export interface V4 {
	readonly stuff: number;
}
export namespace V4 {
	export const codec: Codec<V4> = {
		async read(reader: FormatReader): Promise<V4> {
			return {
				stuff: await StandardCodecs.i32.read(reader),
			};
		},
		async write(writer: FormatWriter, value: V4): Promise<void> {
			await StandardCodecs.i32.write(writer, value.stuff);
		},
	};
}
