package dev.argon.verilization.scala_runtime

import zio.{ZIO, Chunk}
import scala.{Int => SInt}

object Util {
    def mapList[T, U](f: T => U): Chunk[T] => Chunk[U] = x => x.map(f)
    def mapOption[T, U](f: T => U): Option[T] => Option[U] = x => x.map(f)


    object BigIntValue {
        def unapply(n: BigInt): Option[SInt] =
            if(n < SInt.MinValue || n > SInt.MaxValue) None
            else Some(n.toInt)
    }
}
