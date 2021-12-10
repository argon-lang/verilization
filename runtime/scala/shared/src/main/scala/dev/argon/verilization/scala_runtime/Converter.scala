package dev.argon.verilization.scala_runtime

@FunctionalInterface
trait Converter[A, B] {
    def convert(prev: A): B
}

object Converter {
    def identity[A]: Converter[A, A] = new IdentityConverter[A]()
}

