package dev.argon.verilization.java_runtime;

import java.io.IOException;

public interface FormatWriter {
	void writeByte(byte b) throws IOException;
	void writeShort(short s) throws IOException;
	void writeInt(int i) throws IOException;
	void writeLong(long l) throws IOException;
	void writeBytes(byte[] data) throws IOException;
}
