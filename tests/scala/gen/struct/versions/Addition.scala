package struct.versions
sealed abstract class Addition
object Addition {
	final case class V4(
		stuff: scala.Int,
	) extends Addition
	object V4 {
		val codec: dev.argon.verilization.scala_runtime.Codec[V4] = new dev.argon.verilization.scala_runtime.Codec[V4] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V4] =
				for {
					field_stuff <- dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader)
				} yield V4(
					field_stuff,
				)
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V4): zio.ZIO[R, E, Unit] = 
				for {
					_ <- writer.writeInt(value.stuff)
				} yield ()
		}
	}
}
