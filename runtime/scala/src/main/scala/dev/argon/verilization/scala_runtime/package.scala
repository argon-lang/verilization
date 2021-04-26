package dev.argon.verilization

package object scala_runtime {
    type Nat = scala.math.BigInt
    type Int = scala.math.BigInt
    type U8 = scala.Byte
    type I8 = scala.Byte
    type U16 = scala.Short
    type I16 = scala.Short
    type U32 = scala.Int
    type I32 = scala.Int
    type U64 = scala.Long
    type I64 = scala.Long
    type String = scala.Predef.String
    type List[A] = zio.Chunk[A]
    type Option[A] = scala.Option[A]
}