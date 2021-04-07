package enum_.versions
sealed abstract class Referenced
object Referenced {
	sealed abstract class V1 extends Referenced
	object V1 {
		final case class x(x: scala.Int) extends V1
		val codec: dev.argon.verilization.scala_runtime.Codec[V1] = new dev.argon.verilization.scala_runtime.Codec[V1] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V1] =
				dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.read(reader).flatMap {
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(0) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader).map(enum_.versions.Referenced.V1.x.apply)
					case _ => zio.IO.die(new java.lang.RuntimeException("Invalid tag number."))
				}
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V1): zio.ZIO[R, E, Unit] = 
				value match {
					case value: V1.x =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 0)
							_ <- writer.writeInt(value.x)
						} yield ()
				}
		}
	}
	sealed abstract class V2 extends Referenced
	object V2 {
		final case class x(x: scala.Long) extends V2
		def fromV1(prev: V1): V2 =
			enum_.versions.Referenced_Conversions.v1ToV2(prev);
		val codec: dev.argon.verilization.scala_runtime.Codec[V2] = new dev.argon.verilization.scala_runtime.Codec[V2] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V2] =
				dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.read(reader).flatMap {
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(0) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader).map(enum_.versions.Referenced.V2.x.apply)
					case _ => zio.IO.die(new java.lang.RuntimeException("Invalid tag number."))
				}
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V2): zio.ZIO[R, E, Unit] = 
				value match {
					case value: V2.x =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 0)
							_ <- writer.writeLong(value.x)
						} yield ()
				}
		}
	}
	sealed abstract class V3 extends Referenced
	object V3 {
		final case class x(x: scala.Long) extends V3
		def fromV2(prev: V2): V3 =
			prev match {
				case prev: V2.x => V3.x(prev.x)
			}
		val codec: dev.argon.verilization.scala_runtime.Codec[V3] = new dev.argon.verilization.scala_runtime.Codec[V3] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V3] =
				dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.read(reader).flatMap {
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(0) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader).map(enum_.versions.Referenced.V3.x.apply)
					case _ => zio.IO.die(new java.lang.RuntimeException("Invalid tag number."))
				}
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V3): zio.ZIO[R, E, Unit] = 
				value match {
					case value: V3.x =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 0)
							_ <- writer.writeLong(value.x)
						} yield ()
				}
		}
	}
	sealed abstract class V4 extends Referenced
	object V4 {
		final case class x(x: scala.Long) extends V4
		def fromV3(prev: V3): V4 =
			prev match {
				case prev: V3.x => V4.x(prev.x)
			}
		val codec: dev.argon.verilization.scala_runtime.Codec[V4] = new dev.argon.verilization.scala_runtime.Codec[V4] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V4] =
				dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.read(reader).flatMap {
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(0) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader).map(enum_.versions.Referenced.V4.x.apply)
					case _ => zio.IO.die(new java.lang.RuntimeException("Invalid tag number."))
				}
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V4): zio.ZIO[R, E, Unit] = 
				value match {
					case value: V4.x =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 0)
							_ <- writer.writeLong(value.x)
						} yield ()
				}
		}
	}
}
