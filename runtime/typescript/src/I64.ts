import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";

export type I64 = bigint;

export const codec: Codec<I64> = {
    async read(reader: FormatReader): Promise<I64> {
        return BigInt.asIntN(64, await reader.readU64());
    },

    write(writer: FormatWriter, value: I64): Promise<void> {
        return writer.writeU64(BigInt.asUintN(64, value));
    },
};

export function fromInteger(n: bigint): I64 {
    return BigInt.asIntN(64, n);
}
