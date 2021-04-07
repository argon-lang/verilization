package struct.versions;
public abstract class Referenced {
	private Referenced() {}
	public static final class V1 extends Referenced {
		public V1(
			int x
		) {
			this.x = x;
		}
		public final int x;
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V1> {
			@Override
			public V1 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				return new V1(
					reader.readInt()
				);
			}
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V1 value) throws java.io.IOException {
				writer.writeInt(value.x);
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V1> codec = new CodecImpl();
	}
	public static final class V2 extends Referenced {
		public V2(
			long x
		) {
			this.x = x;
		}
		public final long x;
		public static V2 fromV1(V1 prev) {
			return struct.versions.Referenced_Conversions.v1ToV2(prev);
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V2> {
			@Override
			public V2 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				return new V2(
					reader.readLong()
				);
			}
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V2 value) throws java.io.IOException {
				writer.writeLong(value.x);
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V2> codec = new CodecImpl();
	}
	public static final class V3 extends Referenced {
		public V3(
			long x
		) {
			this.x = x;
		}
		public final long x;
		public static V3 fromV2(V2 prev) {
			return new V3(
				prev.x
			);
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V3> {
			@Override
			public V3 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				return new V3(
					reader.readLong()
				);
			}
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V3 value) throws java.io.IOException {
				writer.writeLong(value.x);
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V3> codec = new CodecImpl();
	}
	public static final class V4 extends Referenced {
		public V4(
			long x
		) {
			this.x = x;
		}
		public final long x;
		public static V4 fromV3(V3 prev) {
			return new V4(
				prev.x
			);
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V4> {
			@Override
			public V4 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				return new V4(
					reader.readLong()
				);
			}
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V4 value) throws java.io.IOException {
				writer.writeLong(value.x);
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V4> codec = new CodecImpl();
	}
}
