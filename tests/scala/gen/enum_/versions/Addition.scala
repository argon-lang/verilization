package enum_.versions
sealed abstract class Addition
object Addition {
	sealed abstract class V4 extends Addition
	object V4 {
		final case class stuff(stuff: scala.Int) extends V4
		val codec: dev.argon.verilization.scala_runtime.Codec[V4] = new dev.argon.verilization.scala_runtime.Codec[V4] {
			override def read[R, E](reader: dev.argon.verilization.scala_runtime.FormatReader[R, E]): zio.ZIO[R, E, V4] =
				dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.read(reader).flatMap {
					case dev.argon.verilization.scala_runtime.Util.BigIntValue(0) =>
						dev.argon.verilization.scala_runtime.StandardCodecs.i32Codec.read(reader).map(enum_.versions.Addition.V4.stuff.apply)
					case _ => zio.IO.die(new java.lang.RuntimeException("Invalid tag number."))
				}
			override def write[R, E](writer: dev.argon.verilization.scala_runtime.FormatWriter[R, E], value: V4): zio.ZIO[R, E, Unit] = 
				value match {
					case value: V4.stuff =>
						for {
							_ <- dev.argon.verilization.scala_runtime.StandardCodecs.natCodec.write(writer, 0)
							_ <- writer.writeInt(value.stuff)
						} yield ()
				}
		}
	}
}
