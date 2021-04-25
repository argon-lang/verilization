package dev.argon.verilization.scala_runtime

final class IdentityConverter[A] extends Converter[A, A] {
    override def convert(prev: A): A = prev
}

