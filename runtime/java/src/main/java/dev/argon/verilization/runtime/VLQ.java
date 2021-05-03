package dev.argon.verilization.runtime;

import java.io.IOException;
import java.math.BigInteger;

abstract class VLQ {
	private VLQ() {}

	private static final class Encoder {
		public Encoder(FormatWriter writer) {
			this.writer = writer;
		}

		private final FormatWriter writer;
		private int outBitIndex = 0;
		private byte currentByte = 0;

		public void putBit(boolean b) throws IOException {
			if(outBitIndex > 6) { // Only use 7 bits, 8th bit is for tag to indicate more data
				writer.writeByte((byte)(currentByte | 0x80));
				outBitIndex = 0;
				currentByte = 0;
			}

			if(b) currentByte |= 1 << outBitIndex;
			outBitIndex += 1;
		}
		
		public void putSign(boolean sign) throws IOException {
			while(outBitIndex != 6) { // Pad out until the sign bit
				putBit(false);
			}

			putBit(sign);
		}

		public void finish() throws IOException {
			writer.writeByte(currentByte);
		}
	}

	public static void encodeVLQ(FormatWriter writer, boolean isSigned, BigInteger n) throws IOException {
		var nBytes = ((isSigned && n.signum() < 0) ? n.add(BigInteger.ONE) : n).abs().toByteArray();

		var encoder = new Encoder(writer);
		int byteIndex = nBytes.length - 1;
		int bitIndex = 0;
		int zeroCount = 0;

		while(byteIndex >= 0) {
			boolean bit = (nBytes[byteIndex] & (1 << bitIndex)) != 0;
			if(bit) {
				for(int i = 0; i < zeroCount; ++i) {
					encoder.putBit(false);
				}
				zeroCount = 0;
				encoder.putBit(true);
			}
			else {
				++zeroCount;
			}

			++bitIndex;
			if(bitIndex > 7) {
				bitIndex = 0;
				--byteIndex;
			}
		}

		if(isSigned) {
			encoder.putSign(n.signum() < 0);
		}

		encoder.finish();
	}

	private static class BigIntegerBuilder {

		private static final int INITIAL_SIZE = 8;
		private byte[] data = new byte[INITIAL_SIZE];
		private int byteIndex = INITIAL_SIZE - 1;
		private int bitIndex = 0;

		public void putBit(boolean b) {
			if(b) data[byteIndex] |= 1 << bitIndex;
			++bitIndex;
			if(bitIndex > 7) {
				bitIndex = 0;
				--byteIndex;

				if(byteIndex < 0) {
					byte[] newData = new byte[data.length * 2];
					for(int i = 0; i < data.length; ++i) {
						newData[data.length + i] = data[i];
					}

					byteIndex += data.length;
				}
			}
		}

		public BigInteger bigInteger(boolean sign) {
			var result = new BigInteger(sign ? -1 : 1, data);
			if(sign) {
				return result.subtract(BigInteger.ONE);
			}
			else {
				return result;
			}
		}
	}

	public static BigInteger decodeVLQ(FormatReader reader, boolean isSigned) throws IOException {

		var builder = new BigIntegerBuilder();
		
		byte b = reader.readByte();
		while((b & 0x80) != 0) {
			for(int i = 0; i < 7; ++i) {
				builder.putBit((b & (1 << i)) != 0);
			}
			b = reader.readByte();
		}


		for(int i = 0; i < (isSigned ? 6 : 7); ++i) {
			builder.putBit((b & (1 << i)) != 0);
		}

		boolean sign = isSigned && (b & 0x40) != 0;

		return builder.bigInteger(sign);
	}
}
