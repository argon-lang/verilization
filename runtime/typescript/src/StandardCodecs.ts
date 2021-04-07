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

export const u8list: Codec<Uint8Array> = {
    async read(reader: FormatReader): Promise<Uint8Array> {
        const length = await nat.read(reader);
        if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
            throw new Error("Length of array too large");
        }

        const data = await reader.readBytes(Number(length));
        return data;
    },

    async write(writer: FormatWriter, value: Uint8Array): Promise<void> {
        await nat.write(writer, BigInt(value.length));
        await writer.writeBytes(value);
    },
};

export const i8list: Codec<Int8Array> = {
    async read(reader: FormatReader): Promise<Int8Array> {
        return new Int8Array((await u8list.read(reader)).buffer);
    },

    async write(writer: FormatWriter, value: Int8Array): Promise<void> {
        return u8list.write(writer, new Uint8Array(value.buffer));
    },
};

export const u16list: Codec<Uint16Array> = {
    async read(reader: FormatReader): Promise<Uint16Array> {
        const length = await nat.read(reader) * 2n;
        if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
            throw new Error("Length of array too large");
        }

        const data = new Uint16Array(Number(length));
        for(let i = 0; i < data.length; ++i) {
            data[i] = await reader.readU16();
        }
        return data;
    },

    async write(writer: FormatWriter, value: Uint16Array): Promise<void> {
        await nat.write(writer, BigInt(value.length));
        for(let i = 0; i < value.length; ++i) {
            await writer.writeU16(value[i]);
        }
    },
};

export const i16list: Codec<Int16Array> = {
    async read(reader: FormatReader): Promise<Int16Array> {
        return new Int16Array((await u16list.read(reader)).buffer);
    },

    async write(writer: FormatWriter, value: Int16Array): Promise<void> {
        return u16list.write(writer, new Uint16Array(value.buffer));
    },
};

export const u32list: Codec<Uint32Array> = {
    async read(reader: FormatReader): Promise<Uint32Array> {
        const length = await nat.read(reader) * 4n;
        if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
            throw new Error("Length of array too large");
        }

        const data = new Uint32Array(Number(length));
        for(let i = 0; i < data.length; ++i) {
            data[i] = await reader.readU32();
        }
        return data;
    },

    async write(writer: FormatWriter, value: Uint32Array): Promise<void> {
        await nat.write(writer, BigInt(value.length));
        for(let i = 0; i < value.length; ++i) {
            await writer.writeU32(value[i]);
        }
    },
};

export const i32list: Codec<Int32Array> = {
    async read(reader: FormatReader): Promise<Int32Array> {
        return new Int32Array((await u32list.read(reader)).buffer);
    },

    async write(writer: FormatWriter, value: Int32Array): Promise<void> {
        return u32list.write(writer, new Uint32Array(value.buffer));
    },
};

export const u64list: Codec<BigUint64Array> = {
    async read(reader: FormatReader): Promise<BigUint64Array> {
        const length = await nat.read(reader) * 8n;
        if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
            throw new Error("Length of array too large");
        }

        const data = new BigUint64Array(Number(length));
        for(let i = 0; i < data.length; ++i) {
            data[i] = await reader.readU64();
        }
        return data;
    },

    async write(writer: FormatWriter, value: BigUint64Array): Promise<void> {
        await nat.write(writer, BigInt(value.length));
        for(let i = 0; i < value.length; ++i) {
            await writer.writeU64(value[i]);
        }
    },
};

export const i64list: Codec<BigInt64Array> = {
    async read(reader: FormatReader): Promise<BigInt64Array> {
        return new BigInt64Array((await u64list.read(reader)).buffer);
    },

    async write(writer: FormatWriter, value: BigInt64Array): Promise<void> {
        return u64list.write(writer, new BigUint64Array(value.buffer));
    },
};

export function list<T>(elementCodec: Codec<T>): Codec<ReadonlyArray<T>> {
    return {
        async read(reader: FormatReader): Promise<ReadonlyArray<T>> {
            const length = await nat.read(reader);
            if(length > BigInt(Number.MAX_SAFE_INTEGER)) {
                throw new Error("Length of array too large");
            }

            const data = new Array<T>(Number(length));
            for(let i = 0; i < data.length; ++i) {
                data[i] = await elementCodec.read(reader);
            }
    
            return data;
        },

        async write(writer: FormatWriter, value: ReadonlyArray<T>): Promise<void> {
            await nat.write(writer, BigInt(value.length));
            for(let i = 0; i < value.length; ++i) {
                await elementCodec.write(writer, value[i]);
            }
        },
    };
}

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


