package enum_.versions;
public abstract class Addition {
	private Addition() {}
	public static abstract class V4 extends Addition {
		private V4() {}
		public static final class stuff extends V4 {
			public stuff(int stuff) {
				this.stuff = stuff;
			}
			public final int stuff;
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V4> {
			@Override
			public V4 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				java.math.BigInteger tag = dev.argon.verilization.java_runtime.StandardCodecs.natCodec.read(reader);
				switch(tag.intValue()) {
					case 0:
						return new V4.stuff(reader.readInt());
					default:
						throw new java.io.IOException("Invalid tag number.");
				}
			}
			@Override
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V4 value) throws java.io.IOException {
				if(value instanceof V4.stuff) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(0));
					writer.writeInt(((V4.stuff)value).stuff);
				}
				else {
					throw new IllegalArgumentException();
				}
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V4> codec = new CodecImpl();
	}
}
