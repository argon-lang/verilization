import {FormatReader, FormatWriter} from "@verilization/runtime";

export class MemoryFormatReader implements FormatReader {
    constructor(private readonly data: Uint8Array) {}
    private index: number = 0;

    async readU8(): Promise<number> {
        if(this.index >= this.data.length) {
            throw new Error("End of Stream");
        }

        const value = this.data[this.index];
        ++this.index;
        return value;
    }

    async readU16(): Promise<number> {
        const lower = await this.readU8();
        const upper = await this.readU8();

        return (upper << 8) | lower;
    }
    
    async readU32(): Promise<number> {
        const lower = await this.readU16();
        const upper = await this.readU16();

        return ((upper << 16) | lower) >>> 0;
    }
    
    async readU64(): Promise<bigint> {
        const lower = await this.readU32();
        const upper = await this.readU32();

        return (BigInt(upper) << 32n) | BigInt(lower);
    }

    async readBytes(count: number): Promise<Uint8Array> {
        const arr = new Uint8Array(count);
        for(let i = 0; i < arr.length; ++i) {
            arr[i] = await this.readU8();
        }
        return arr;
    }

    isEOF(): boolean {
        return this.index >= this.data.length;
    }

}


export class MemoryFormatWriter implements FormatWriter {
    private readonly data: number[] = [];


    async writeU8(b: number): Promise<void> {
        this.data.push(b);
    }
    
    async writeU16(s: number): Promise<void> {
        await this.writeU8(s & 0xFF);
        await this.writeU8(s >>> 8);
    }

    async writeU32(i: number): Promise<void> {
        await this.writeU16(i & 0xFFFF);
        await this.writeU16(i >>> 16);
    }
    
    async writeU64(l: bigint): Promise<void> {
        await this.writeU32(Number(l & 0xFFFFFFFFn) >>> 0);
        await this.writeU32(Number(l >> 32n) >>> 0);
    }
    
    async writeBytes(data: Uint8Array): Promise<void> {
        for(let i = 0; i < data.length; ++i) {
            await this.writeU8(data[i]);
        }
    }

    toUint8Array(): Uint8Array {
        return new Uint8Array(this.data);
    }


}

