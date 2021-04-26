import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";

export type U64 = bigint;

export const codec: Codec<U64> = {
    read(reader: FormatReader): Promise<U64> {
        return reader.readU64();
    },

    write(writer: FormatWriter, value: U64): Promise<void> {
        return writer.writeU64(value);
    },
};
