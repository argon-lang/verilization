package dev.argon.verilization.runtime;

import java.io.IOException;

public interface Codec<T> {	
	T read(FormatReader reader) throws IOException;
	void write(FormatWriter writer, T value) throws IOException;
}
