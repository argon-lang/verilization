import { Codec } from "./Codec.js";
import { FormatReader, FormatWriter } from "./FormatIO.js";
import { encodeVLQ, decodeVLQ } from "./VLQ.js";

export const nat: Codec<bigint> = {
    read(reader: FormatReader): Promise<bigint> {
        return decodeVLQ(reader, false);
    },

    write(writer: FormatWriter, value: bigint): Promise<void> {
        return encodeVLQ(writer, false, value);
    },
};

export const int: Codec<bigint> = {
    read(reader: FormatReader): Promise<bigint> {
        return decodeVLQ(reader, true);
    },

    write(writer: FormatWriter, value: bigint): Promise<void> {
        return encodeVLQ(writer, true, value);
    },
};

export const u8: Codec<number> = {
    read(reader: FormatReader): Promise<number> {
        return reader.readU8();
    },

    write(writer: FormatWriter, value: number): Promise<void> {
        return writer.writeU8(value);
    },
};

export const i8: Codec<number> = {
    async read(reader: FormatReader): Promise<number> {
        return (await reader.readU8() << 24) >> 24;
    },

    write(writer: FormatWriter, value: number): Promise<void> {
        return writer.writeU8(value & 0xFF);
    },
};

export const u16: Codec<number> = {
    read(reader: FormatReader): Promise<number> {
        return reader.readU16();
    },

    write(writer: FormatWriter, value: number): Promise<void> {
        return writer.writeU16(value);
    },
};

export const i16: Codec<number> = {
    async read(reader: FormatReader): Promise<number> {
        return (await reader.readU16() << 16) >> 16;
    },

    write(writer: FormatWriter, value: number): Promise<void> {
        return writer.writeU8(value & 0xFFFF);
    },
};

export const u32: Codec<number> = {
    read(reader: FormatReader): Promise<number> {
        return reader.readU32();
    },

    write(writer: FormatWriter, value: number): Promise<void> {
        return writer.writeU32(value);
    },
};

export const i32: Codec<number> = {
    async read(reader: FormatReader): Promise<number> {
        return await reader.readU32() | 0;
    },

    write(writer: FormatWriter, value: number): Promise<void> {
        return writer.writeU32(value >>> 0);
    },
};

export const u64: Codec<bigint> = {
    read(reader: FormatReader): Promise<bigint> {
        return reader.readU64();
    },

    write(writer: FormatWriter, value: bigint): Promise<void> {
        return writer.writeU64(value);
    },
};

export const i64: Codec<bigint> = {
    async read(reader: FormatReader): Promise<bigint> {
        return BigInt.asIntN(64, await reader.readU64());
    },

    write(writer: FormatWriter, value: bigint): Promise<void> {
        return writer.writeU64(BigInt.asUintN(64, value));
    },
};

export const string: Codec<string> = {
    async read(reader: FormatReader): Promise<string> {
        const length = await nat.read(reader);
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

        await nat.write(writer, BigInt(data.length));
        await writer.writeBytes(data);
    },
};

export function option<T>(elementCodec: Codec<T>): Codec<{ value: T } | null> {
    return {
        async read(reader: FormatReader): Promise<{ value: T } | null> {
            const present = await reader.readU8();

            if(present !== 0) {
                const value = await elementCodec.read(reader);
                return { value };
            }
            else {
                return null;
            }
        },

        async write(writer: FormatWriter, value: { value: T } | null): Promise<void> {
            if(value !== null) {
                await writer.writeU8(1);
                await elementCodec.write(writer, value.value);
            }
            else {
                await writer.writeU8(0);
            }
        },
    };
}


