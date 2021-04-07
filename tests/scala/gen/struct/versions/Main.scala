package struct.versions
sealed abstract class Main
object Main {
	final case class V1(
		n: scala.Int,
		m: scala.Long,
		r: struct.versions.Referenced.V1,
	) extends Main
	object V1 {
		val codec: dev.argon.verilization.scala_runtime.Codec[V1] = new dev.argon.verilization.scala_runtime.Codec[V1] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V1] =
				for {
					field_n <- dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader)
					field_m <- dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader)
					field_r <- struct.versions.Referenced.V1.codec.read(reader)
				} yield V1(
					field_n,
					field_m,
					field_r,
				)
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V1): zio.ZIO[R, E, Unit] = 
				for {
					_ <- writer.writeInt(value.n)
					_ <- writer.writeLong(value.m)
					_ <- struct.versions.Referenced.V1.codec.write(writer, value.r)
				} yield ()
		}
	}
	final case class V2(
		n: scala.Int,
		m: scala.Long,
		r: struct.versions.Referenced.V2,
	) extends Main
	object V2 {
		def fromV1(prev: V1): V2 =
			V2(
				prev.n,
				prev.m,
				struct.versions.Referenced.V2.fromV1(prev.r),
			)
		val codec: dev.argon.verilization.scala_runtime.Codec[V2] = new dev.argon.verilization.scala_runtime.Codec[V2] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V2] =
				for {
					field_n <- dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader)
					field_m <- dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader)
					field_r <- struct.versions.Referenced.V2.codec.read(reader)
				} yield V2(
					field_n,
					field_m,
					field_r,
				)
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V2): zio.ZIO[R, E, Unit] = 
				for {
					_ <- writer.writeInt(value.n)
					_ <- writer.writeLong(value.m)
					_ <- struct.versions.Referenced.V2.codec.write(writer, value.r)
				} yield ()
		}
	}
	final case class V3(
		n: scala.Int,
		m: scala.Long,
		r: struct.versions.Referenced.V3,
	) extends Main
	object V3 {
		def fromV2(prev: V2): V3 =
			V3(
				prev.n,
				prev.m,
				struct.versions.Referenced.V3.fromV2(prev.r),
			)
		val codec: dev.argon.verilization.scala_runtime.Codec[V3] = new dev.argon.verilization.scala_runtime.Codec[V3] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V3] =
				for {
					field_n <- dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader)
					field_m <- dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader)
					field_r <- struct.versions.Referenced.V3.codec.read(reader)
				} yield V3(
					field_n,
					field_m,
					field_r,
				)
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V3): zio.ZIO[R, E, Unit] = 
				for {
					_ <- writer.writeInt(value.n)
					_ <- writer.writeLong(value.m)
					_ <- struct.versions.Referenced.V3.codec.write(writer, value.r)
				} yield ()
		}
	}
	final case class V4(
		n: scala.Int,
		m: scala.Long,
		r: struct.versions.Referenced.V4,
		addition: struct.versions.Addition.V4,
	) extends Main
	object V4 {
		def fromV3(prev: V3): V4 =
			struct.versions.Main_Conversions.v3ToV4(prev);
		val codec: dev.argon.verilization.scala_runtime.Codec[V4] = new dev.argon.verilization.scala_runtime.Codec[V4] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V4] =
				for {
					field_n <- dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader)
					field_m <- dev.argon.verilization.scala_runtime.StandardCodecs.i64Codec.read(reader)
					field_r <- struct.versions.Referenced.V4.codec.read(reader)
					field_addition <- struct.versions.Addition.V4.codec.read(reader)
				} yield V4(
					field_n,
					field_m,
					field_r,
					field_addition,
				)
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V4): zio.ZIO[R, E, Unit] = 
				for {
					_ <- writer.writeInt(value.n)
					_ <- writer.writeLong(value.m)
					_ <- struct.versions.Referenced.V4.codec.write(writer, value.r)
					_ <- struct.versions.Addition.V4.codec.write(writer, value.addition)
				} yield ()
		}
	}
}
