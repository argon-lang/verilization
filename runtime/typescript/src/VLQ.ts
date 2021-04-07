import { FormatWriter, FormatReader } from "./FormatIO.js";

export async function encodeVLQ(encoder: FormatWriter, isSigned: boolean, value: bigint): Promise<void> {
	const result: Array<number> = [];

	const isNeg = value < 0n;
	if(isNeg) {
		value = -value;
	}

	while(value > 0) {
		result.push(Number(value & 0x7Fn));
		value >>= 7n;
	}

	if(result.length === 0) {
		result.push(0);
	}

	if(isSigned) {
		if((result[0] & 0x40) !== 0) {
			result.push(0);
		}
		if(isNeg) result[0] |= 0x40;
	}

	for(let i = 0; i < result.length - 1; ++i) {
		result[i] |= 0x80;
	}

	await encoder.writeBytes(new Uint8Array(result));
}

export async function decodeVLQ(decoder: FormatReader, isSigned: boolean): Promise<bigint> {
		let b = await decoder.readU8();

		let result = 0n;
		let shift = 0n;

		while(true) {
			const isLast = (b & 0x80) === 0;
			if(isLast && isSigned) {
				result |= BigInt(b & 0x3F) << shift;
				if((b & 0x40) !== 0) {
					result = -result;
				}
				return result;
			}
			else {
				result |= BigInt(b & 0x7F) << shift;
				shift += 7n;

				if(isLast) {
					return result;
				}
			}

			
			b = await decoder.readU8();
		}
}
