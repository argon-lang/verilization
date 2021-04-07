package struct.versions
sealed abstract class Referenced
object Referenced {
	final case class V1(
		x: scala.Int,
	) extends Referenced
	object V1 {
		val codec: dev.argon.verilization.scala_runtime.Codec[V1] = new dev.argon.verilization.scala_runtime.Codec[V1] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V1] =
				for {
					field_x <- dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader)
				} yield V1(
					field_x,
				)
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V1): zio.ZIO[R, E, Unit] = 
				for {
					_ <- writer.writeInt(value.x)
				} yield ()
		}
	}
	final case class V2(
		x: scala.Long,
	) extends Referenced
	object V2 {
		def fromV1(prev: V1): V2 =
			struct.versions.Referenced_Conversions.v1ToV2(prev);
		val codec: dev.argon.verilization.scala_runtime.Codec[V2] = new dev.argon.verilization.scala_runtime.Codec[V2] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V2] =
				for {
					field_x <- dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader)
				} yield V2(
					field_x,
				)
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V2): zio.ZIO[R, E, Unit] = 
				for {
					_ <- writer.writeLong(value.x)
				} yield ()
		}
	}
	final case class V3(
		x: scala.Long,
	) extends Referenced
	object V3 {
		def fromV2(prev: V2): V3 =
			V3(
				prev.x,
			)
		val codec: dev.argon.verilization.scala_runtime.Codec[V3] = new dev.argon.verilization.scala_runtime.Codec[V3] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V3] =
				for {
					field_x <- dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader)
				} yield V3(
					field_x,
				)
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V3): zio.ZIO[R, E, Unit] = 
				for {
					_ <- writer.writeLong(value.x)
				} yield ()
		}
	}
	final case class V4(
		x: scala.Long,
	) extends Referenced
	object V4 {
		def fromV3(prev: V3): V4 =
			V4(
				prev.x,
			)
		val codec: dev.argon.verilization.scala_runtime.Codec[V4] = new dev.argon.verilization.scala_runtime.Codec[V4] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V4] =
				for {
					field_x <- dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader)
				} yield V4(
					field_x,
				)
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V4): zio.ZIO[R, E, Unit] = 
				for {
					_ <- writer.writeLong(value.x)
				} yield ()
		}
	}
}
