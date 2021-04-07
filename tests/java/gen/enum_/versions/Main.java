package enum_.versions;
public abstract class Main {
	private Main() {}
	public static abstract class V1 extends Main {
		private V1() {}
		public static final class n extends V1 {
			public n(int n) {
				this.n = n;
			}
			public final int n;
		}
		public static final class m extends V1 {
			public m(long m) {
				this.m = m;
			}
			public final long m;
		}
		public static final class r extends V1 {
			public r(enum_.versions.Referenced.V1 r) {
				this.r = r;
			}
			public final enum_.versions.Referenced.V1 r;
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V1> {
			@Override
			public V1 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				java.math.BigInteger tag = dev.argon.verilization.java_runtime.StandardCodecs.natCodec.read(reader);
				if(tag.compareTo(java.math.BigInteger.valueOf(java.lang.Integer.MAX_VALUE)) > 0) throw new java.lang.ArithmeticException();
				switch(tag.intValue()) {
					case 0:
						return new V1.n(reader.readInt());
					case 1:
						return new V1.m(reader.readLong());
					case 2:
						return new V1.r(enum_.versions.Referenced.V1.codec.read(reader));
					default:
						throw new java.io.IOException("Invalid tag number.");
				}
			}
			@Override
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V1 value) throws java.io.IOException {
				if(value instanceof V1.n) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(0));
					writer.writeInt(((V1.n)value).n);
				}
				else if(value instanceof V1.m) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(1));
					writer.writeLong(((V1.m)value).m);
				}
				else if(value instanceof V1.r) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(2));
					enum_.versions.Referenced.V1.codec.write(writer, ((V1.r)value).r);
				}
				else {
					throw new IllegalArgumentException();
				}
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V1> codec = new CodecImpl();
	}
	public static abstract class V2 extends Main {
		private V2() {}
		public static final class n extends V2 {
			public n(int n) {
				this.n = n;
			}
			public final int n;
		}
		public static final class m extends V2 {
			public m(long m) {
				this.m = m;
			}
			public final long m;
		}
		public static final class r extends V2 {
			public r(enum_.versions.Referenced.V2 r) {
				this.r = r;
			}
			public final enum_.versions.Referenced.V2 r;
		}
		public static V2 fromV1(V1 prev) {
			if(prev instanceof V1.n) {
				return new V2.n(((V1.n)prev).n);
			}
			else if(prev instanceof V1.m) {
				return new V2.m(((V1.m)prev).m);
			}
			else if(prev instanceof V1.r) {
				return new V2.r(enum_.versions.Referenced.V2.V2.fromV1(((V1.r)prev).r));
			}
			else {
				throw new IllegalArgumentException();
			}
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V2> {
			@Override
			public V2 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				java.math.BigInteger tag = dev.argon.verilization.java_runtime.StandardCodecs.natCodec.read(reader);
				if(tag.compareTo(java.math.BigInteger.valueOf(java.lang.Integer.MAX_VALUE)) > 0) throw new java.lang.ArithmeticException();
				switch(tag.intValue()) {
					case 0:
						return new V2.n(reader.readInt());
					case 1:
						return new V2.m(reader.readLong());
					case 2:
						return new V2.r(enum_.versions.Referenced.V2.codec.read(reader));
					default:
						throw new java.io.IOException("Invalid tag number.");
				}
			}
			@Override
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V2 value) throws java.io.IOException {
				if(value instanceof V2.n) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(0));
					writer.writeInt(((V2.n)value).n);
				}
				else if(value instanceof V2.m) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(1));
					writer.writeLong(((V2.m)value).m);
				}
				else if(value instanceof V2.r) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(2));
					enum_.versions.Referenced.V2.codec.write(writer, ((V2.r)value).r);
				}
				else {
					throw new IllegalArgumentException();
				}
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V2> codec = new CodecImpl();
	}
	public static abstract class V3 extends Main {
		private V3() {}
		public static final class n extends V3 {
			public n(int n) {
				this.n = n;
			}
			public final int n;
		}
		public static final class m extends V3 {
			public m(long m) {
				this.m = m;
			}
			public final long m;
		}
		public static final class r extends V3 {
			public r(enum_.versions.Referenced.V3 r) {
				this.r = r;
			}
			public final enum_.versions.Referenced.V3 r;
		}
		public static V3 fromV2(V2 prev) {
			if(prev instanceof V2.n) {
				return new V3.n(((V2.n)prev).n);
			}
			else if(prev instanceof V2.m) {
				return new V3.m(((V2.m)prev).m);
			}
			else if(prev instanceof V2.r) {
				return new V3.r(enum_.versions.Referenced.V3.V3.fromV2(((V2.r)prev).r));
			}
			else {
				throw new IllegalArgumentException();
			}
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V3> {
			@Override
			public V3 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				java.math.BigInteger tag = dev.argon.verilization.java_runtime.StandardCodecs.natCodec.read(reader);
				if(tag.compareTo(java.math.BigInteger.valueOf(java.lang.Integer.MAX_VALUE)) > 0) throw new java.lang.ArithmeticException();
				switch(tag.intValue()) {
					case 0:
						return new V3.n(reader.readInt());
					case 1:
						return new V3.m(reader.readLong());
					case 2:
						return new V3.r(enum_.versions.Referenced.V3.codec.read(reader));
					default:
						throw new java.io.IOException("Invalid tag number.");
				}
			}
			@Override
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V3 value) throws java.io.IOException {
				if(value instanceof V3.n) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(0));
					writer.writeInt(((V3.n)value).n);
				}
				else if(value instanceof V3.m) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(1));
					writer.writeLong(((V3.m)value).m);
				}
				else if(value instanceof V3.r) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(2));
					enum_.versions.Referenced.V3.codec.write(writer, ((V3.r)value).r);
				}
				else {
					throw new IllegalArgumentException();
				}
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V3> codec = new CodecImpl();
	}
	public static abstract class V4 extends Main {
		private V4() {}
		public static final class n extends V4 {
			public n(int n) {
				this.n = n;
			}
			public final int n;
		}
		public static final class m extends V4 {
			public m(long m) {
				this.m = m;
			}
			public final long m;
		}
		public static final class r extends V4 {
			public r(enum_.versions.Referenced.V4 r) {
				this.r = r;
			}
			public final enum_.versions.Referenced.V4 r;
		}
		public static final class addition extends V4 {
			public addition(enum_.versions.Addition.V4 addition) {
				this.addition = addition;
			}
			public final enum_.versions.Addition.V4 addition;
		}
		public static V4 fromV3(V3 prev) {
			return enum_.versions.Main_Conversions.v3ToV4(prev);
		}
		private static final class CodecImpl implements dev.argon.verilization.java_runtime.Codec<V4> {
			@Override
			public V4 read(dev.argon.verilization.java_runtime.FormatReader reader) throws java.io.IOException {
				java.math.BigInteger tag = dev.argon.verilization.java_runtime.StandardCodecs.natCodec.read(reader);
				if(tag.compareTo(java.math.BigInteger.valueOf(java.lang.Integer.MAX_VALUE)) > 0) throw new java.lang.ArithmeticException();
				switch(tag.intValue()) {
					case 0:
						return new V4.n(reader.readInt());
					case 1:
						return new V4.m(reader.readLong());
					case 2:
						return new V4.r(enum_.versions.Referenced.V4.codec.read(reader));
					case 3:
						return new V4.addition(enum_.versions.Addition.V4.codec.read(reader));
					default:
						throw new java.io.IOException("Invalid tag number.");
				}
			}
			@Override
			public void write(dev.argon.verilization.java_runtime.FormatWriter writer, V4 value) throws java.io.IOException {
				if(value instanceof V4.n) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(0));
					writer.writeInt(((V4.n)value).n);
				}
				else if(value instanceof V4.m) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(1));
					writer.writeLong(((V4.m)value).m);
				}
				else if(value instanceof V4.r) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(2));
					enum_.versions.Referenced.V4.codec.write(writer, ((V4.r)value).r);
				}
				else if(value instanceof V4.addition) {
					dev.argon.verilization.java_runtime.StandardCodecs.natCodec.write(writer, java.math.BigInteger.valueOf(3));
					enum_.versions.Addition.V4.codec.write(writer, ((V4.addition)value).addition);
				}
				else {
					throw new IllegalArgumentException();
				}
			}
		}
		public static final dev.argon.verilization.java_runtime.Codec<V4> codec = new CodecImpl();
	}
}
