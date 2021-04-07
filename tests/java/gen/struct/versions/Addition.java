package struct.versions;
public abstract class Addition {
	private Addition() {}
	public static final class V4 extends Addition {
		public V4(
			int stuff
		) {
			this.stuff = stuff;
		}
		public final int stuff;
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V4> {
			@Override
			public V4 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				return new V4(
					reader.readInt()
				);
			}
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V4 value) throws java.io.IOException {
				writer.writeInt(value.stuff);
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V4> codec = new CodecImpl();
	}
}
