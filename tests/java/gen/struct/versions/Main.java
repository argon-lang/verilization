package struct.versions;
public abstract class Main {
	private Main() {}
	public static final class V1 extends Main {
		public V1(
			int n,
			long m,
			struct.versions.Referenced.V1 r
		) {
			this.n = n;
			this.m = m;
			this.r = r;
		}
		public final int n;
		public final long m;
		public final struct.versions.Referenced.V1 r;
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V1> {
			@Override
			public V1 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				return new V1(
					reader.readInt(),
					reader.readLong(),
					struct.versions.Referenced.V1.codec.read(reader)
				);
			}
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V1 value) throws java.io.IOException {
				writer.writeInt(value.n);
				writer.writeLong(value.m);
				struct.versions.Referenced.V1.codec.write(writer, value.r);
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V1> codec = new CodecImpl();
	}
	public static final class V2 extends Main {
		public V2(
			int n,
			long m,
			struct.versions.Referenced.V2 r
		) {
			this.n = n;
			this.m = m;
			this.r = r;
		}
		public final int n;
		public final long m;
		public final struct.versions.Referenced.V2 r;
		public static V2 fromV1(V1 prev) {
			return new V2(
				prev.n,
				prev.m,
				struct.versions.Referenced.V2.V2.fromV1(prev.r)
			);
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V2> {
			@Override
			public V2 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				return new V2(
					reader.readInt(),
					reader.readLong(),
					struct.versions.Referenced.V2.codec.read(reader)
				);
			}
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V2 value) throws java.io.IOException {
				writer.writeInt(value.n);
				writer.writeLong(value.m);
				struct.versions.Referenced.V2.codec.write(writer, value.r);
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V2> codec = new CodecImpl();
	}
	public static final class V3 extends Main {
		public V3(
			int n,
			long m,
			struct.versions.Referenced.V3 r
		) {
			this.n = n;
			this.m = m;
			this.r = r;
		}
		public final int n;
		public final long m;
		public final struct.versions.Referenced.V3 r;
		public static V3 fromV2(V2 prev) {
			return new V3(
				prev.n,
				prev.m,
				struct.versions.Referenced.V3.V3.fromV2(prev.r)
			);
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V3> {
			@Override
			public V3 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				return new V3(
					reader.readInt(),
					reader.readLong(),
					struct.versions.Referenced.V3.codec.read(reader)
				);
			}
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V3 value) throws java.io.IOException {
				writer.writeInt(value.n);
				writer.writeLong(value.m);
				struct.versions.Referenced.V3.codec.write(writer, value.r);
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V3> codec = new CodecImpl();
	}
	public static final class V4 extends Main {
		public V4(
			int n,
			long m,
			struct.versions.Referenced.V4 r,
			struct.versions.Addition.V4 addition
		) {
			this.n = n;
			this.m = m;
			this.r = r;
			this.addition = addition;
		}
		public final int n;
		public final long m;
		public final struct.versions.Referenced.V4 r;
		public final struct.versions.Addition.V4 addition;
		public static V4 fromV3(V3 prev) {
			return struct.versions.Main_Conversions.v3ToV4(prev);
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V4> {
			@Override
			public V4 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				return new V4(
					reader.readInt(),
					reader.readLong(),
					struct.versions.Referenced.V4.codec.read(reader),
					struct.versions.Addition.V4.codec.read(reader)
				);
			}
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V4 value) throws java.io.IOException {
				writer.writeInt(value.n);
				writer.writeLong(value.m);
				struct.versions.Referenced.V4.codec.write(writer, value.r);
				struct.versions.Addition.V4.codec.write(writer, value.addition);
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V4> codec = new CodecImpl();
	}
}
