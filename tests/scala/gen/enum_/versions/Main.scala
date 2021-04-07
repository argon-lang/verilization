package enum_.versions
sealed abstract class Main
object Main {
	sealed abstract class V1 extends Main
	object V1 {
		final case class n(n: scala.Int) extends V1
		final case class m(m: scala.Long) extends V1
		final case class r(r: enum_.versions.Referenced.V1) extends V1
		val codec: dev.argon.verilization.scala_runtime.Codec[V1] = new dev.argon.verilization.scala_runtime.Codec[V1] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V1] =
				dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.read(reader).flatMap {
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(0) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader).map(enum_.versions.Main.V1.n.apply)
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(1) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader).map(enum_.versions.Main.V1.m.apply)
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(2) =>
						enum_.versions.Referenced.V1.codec.read(reader).map(enum_.versions.Main.V1.r.apply)
					case _ => zio.IO.die(new java.lang.RuntimeException("Invalid tag number."))
				}
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V1): zio.ZIO[R, E, Unit] = 
				value match {
					case value: V1.n =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 0)
							_ <- writer.writeInt(value.n)
						} yield ()
					case value: V1.m =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 1)
							_ <- writer.writeLong(value.m)
						} yield ()
					case value: V1.r =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 2)
							_ <- enum_.versions.Referenced.V1.codec.write(writer, value.r)
						} yield ()
				}
		}
	}
	sealed abstract class V2 extends Main
	object V2 {
		final case class n(n: scala.Int) extends V2
		final case class m(m: scala.Long) extends V2
		final case class r(r: enum_.versions.Referenced.V2) extends V2
		def fromV1(prev: V1): V2 =
			prev match {
				case prev: V1.n => V2.n(prev.n)
				case prev: V1.m => V2.m(prev.m)
				case prev: V1.r => V2.r(enum_.versions.Referenced.V2.fromV1(prev.r))
			}
		val codec: dev.argon.verilization.scala_runtime.Codec[V2] = new dev.argon.verilization.scala_runtime.Codec[V2] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V2] =
				dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.read(reader).flatMap {
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(0) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader).map(enum_.versions.Main.V2.n.apply)
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(1) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader).map(enum_.versions.Main.V2.m.apply)
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(2) =>
						enum_.versions.Referenced.V2.codec.read(reader).map(enum_.versions.Main.V2.r.apply)
					case _ => zio.IO.die(new java.lang.RuntimeException("Invalid tag number."))
				}
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V2): zio.ZIO[R, E, Unit] = 
				value match {
					case value: V2.n =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 0)
							_ <- writer.writeInt(value.n)
						} yield ()
					case value: V2.m =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 1)
							_ <- writer.writeLong(value.m)
						} yield ()
					case value: V2.r =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 2)
							_ <- enum_.versions.Referenced.V2.codec.write(writer, value.r)
						} yield ()
				}
		}
	}
	sealed abstract class V3 extends Main
	object V3 {
		final case class n(n: scala.Int) extends V3
		final case class m(m: scala.Long) extends V3
		final case class r(r: enum_.versions.Referenced.V3) extends V3
		def fromV2(prev: V2): V3 =
			prev match {
				case prev: V2.n => V3.n(prev.n)
				case prev: V2.m => V3.m(prev.m)
				case prev: V2.r => V3.r(enum_.versions.Referenced.V3.fromV2(prev.r))
			}
		val codec: dev.argon.verilization.scala_runtime.Codec[V3] = new dev.argon.verilization.scala_runtime.Codec[V3] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V3] =
				dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.read(reader).flatMap {
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(0) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader).map(enum_.versions.Main.V3.n.apply)
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(1) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader).map(enum_.versions.Main.V3.m.apply)
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(2) =>
						enum_.versions.Referenced.V3.codec.read(reader).map(enum_.versions.Main.V3.r.apply)
					case _ => zio.IO.die(new java.lang.RuntimeException("Invalid tag number."))
				}
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V3): zio.ZIO[R, E, Unit] = 
				value match {
					case value: V3.n =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 0)
							_ <- writer.writeInt(value.n)
						} yield ()
					case value: V3.m =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 1)
							_ <- writer.writeLong(value.m)
						} yield ()
					case value: V3.r =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 2)
							_ <- enum_.versions.Referenced.V3.codec.write(writer, value.r)
						} yield ()
				}
		}
	}
	sealed abstract class V4 extends Main
	object V4 {
		final case class n(n: scala.Int) extends V4
		final case class m(m: scala.Long) extends V4
		final case class r(r: enum_.versions.Referenced.V4) extends V4
		final case class addition(addition: enum_.versions.Addition.V4) extends V4
		def fromV3(prev: V3): V4 =
			enum_.versions.Main_Conversions.v3ToV4(prev);
		val codec: dev.argon.verilization.scala_runtime.Codec[V4] = new dev.argon.verilization.scala_runtime.Codec[V4] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V4] =
				dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.read(reader).flatMap {
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(0) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader).map(enum_.versions.Main.V4.n.apply)
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(1) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader).map(enum_.versions.Main.V4.m.apply)
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(2) =>
						enum_.versions.Referenced.V4.codec.read(reader).map(enum_.versions.Main.V4.r.apply)
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(3) =>
						enum_.versions.Addition.V4.codec.read(reader).map(enum_.versions.Main.V4.addition.apply)
					case _ => zio.IO.die(new java.lang.RuntimeException("Invalid tag number."))
				}
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V4): zio.ZIO[R, E, Unit] = 
				value match {
					case value: V4.n =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 0)
							_ <- writer.writeInt(value.n)
						} yield ()
					case value: V4.m =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 1)
							_ <- writer.writeLong(value.m)
						} yield ()
					case value: V4.r =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 2)
							_ <- enum_.versions.Referenced.V4.codec.write(writer, value.r)
						} yield ()
					case value: V4.addition =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 3)
							_ <- enum_.versions.Addition.V4.codec.write(writer, value.addition)
						} yield ()
				}
		}
	}
}
