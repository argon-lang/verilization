package dev.argon.verilization.java_runtime;

import java.io.IOException;

public interface FormatReader {
	byte readByte() throws IOException;
	short readShort() throws IOException;
	int readInt() throws IOException;
	long readLong() throws IOException;
	byte[] readBytes(int count) throws IOException;
}
