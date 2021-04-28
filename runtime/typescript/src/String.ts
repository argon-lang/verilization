import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";
import { codec as natCodec } from "./Nat.js";

export type String = string;

export function fromString(s: string): string {
    return s;
}

export const codec: Codec<string> = {
    async read(reader: FormatReader): Promise<string> {
        const length = await natCodec.read(reader);
        if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
            throw new Error("Length of string too large");
        }

        const data = await reader.readBytes(Number(length));
        const utf8Decoder = new TextDecoder('utf-8');
        return utf8Decoder.decode(data);
    },

    async write(writer: FormatWriter, value: string): Promise<void> {
        const utf8Encoder = new TextEncoder();
        const data = utf8Encoder.encode(value);

        await natCodec.write(writer, BigInt(data.length));
        await writer.writeBytes(data);
    },
};



