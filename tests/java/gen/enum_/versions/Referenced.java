package enum_.versions;
public abstract class Referenced {
	private Referenced() {}
	public static abstract class V1 extends Referenced {
		private V1() {}
		public static final class x extends V1 {
			public x(int x) {
				this.x = x;
			}
			public final int x;
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V1> {
			@Override
			public V1 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				java.math.BigInteger tag = dev.argon.verilization.java_runtime.StandardCodecs.natCodec.read(reader);
				switch(tag.intValue()) {
					case 0:
						return new V1.x(reader.readInt());
					default:
						throw new java.io.IOException("Invalid tag number.");
				}
			}
			@Override
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V1 value) throws java.io.IOException {
				if(value instanceof V1.x) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(0));
					writer.writeInt(((V1.x)value).x);
				}
				else {
					throw new IllegalArgumentException();
				}
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V1> codec = new CodecImpl();
	}
	public static abstract class V2 extends Referenced {
		private V2() {}
		public static final class x extends V2 {
			public x(long x) {
				this.x = x;
			}
			public final long x;
		}
		public static V2 fromV1(V1 prev) {
			return enum_.versions.Referenced_Conversions.v1ToV2(prev);
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V2> {
			@Override
			public V2 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				java.math.BigInteger tag = dev.argon.verilization.java_runtime.StandardCodecs.natCodec.read(reader);
				switch(tag.intValue()) {
					case 0:
						return new V2.x(reader.readLong());
					default:
						throw new java.io.IOException("Invalid tag number.");
				}
			}
			@Override
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V2 value) throws java.io.IOException {
				if(value instanceof V2.x) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(0));
					writer.writeLong(((V2.x)value).x);
				}
				else {
					throw new IllegalArgumentException();
				}
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V2> codec = new CodecImpl();
	}
	public static abstract class V3 extends Referenced {
		private V3() {}
		public static final class x extends V3 {
			public x(long x) {
				this.x = x;
			}
			public final long x;
		}
		public static V3 fromV2(V2 prev) {
			if(prev instanceof V2.x) {
				return new V3.x(((V2.x)prev).x);
			}
			else {
				throw new IllegalArgumentException();
			}
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V3> {
			@Override
			public V3 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				java.math.BigInteger tag = dev.argon.verilization.java_runtime.StandardCodecs.natCodec.read(reader);
				switch(tag.intValue()) {
					case 0:
						return new V3.x(reader.readLong());
					default:
						throw new java.io.IOException("Invalid tag number.");
				}
			}
			@Override
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V3 value) throws java.io.IOException {
				if(value instanceof V3.x) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(0));
					writer.writeLong(((V3.x)value).x);
				}
				else {
					throw new IllegalArgumentException();
				}
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V3> codec = new CodecImpl();
	}
	public static abstract class V4 extends Referenced {
		private V4() {}
		public static final class x extends V4 {
			public x(long x) {
				this.x = x;
			}
			public final long x;
		}
		public static V4 fromV3(V3 prev) {
			if(prev instanceof V3.x) {
				return new V4.x(((V3.x)prev).x);
			}
			else {
				throw new IllegalArgumentException();
			}
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V4> {
			@Override
			public V4 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				java.math.BigInteger tag = dev.argon.verilization.java_runtime.StandardCodecs.natCodec.read(reader);
				switch(tag.intValue()) {
					case 0:
						return new V4.x(reader.readLong());
					default:
						throw new java.io.IOException("Invalid tag number.");
				}
			}
			@Override
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V4 value) throws java.io.IOException {
				if(value instanceof V4.x) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(0));
					writer.writeLong(((V4.x)value).x);
				}
				else {
					throw new IllegalArgumentException();
				}
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V4> codec = new CodecImpl();
	}
}
