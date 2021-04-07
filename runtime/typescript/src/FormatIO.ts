
export interface FormatWriter {
    writeU8(b: number): Promise<void>;
    writeU16(s: number): Promise<void>;
    writeU32(i: number): Promise<void>;
    writeU64(l: bigint): Promise<void>;
    writeBytes(data: Uint8Array): Promise<void>;
}

export interface FormatReader {
    readU8(): Promise<number>;
    readU16(): Promise<number>;
    readU32(): Promise<number>;
    readU64(): Promise<bigint>;
    readBytes(count: number): Promise<Uint8Array>;
}
