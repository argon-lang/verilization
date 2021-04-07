package dev.argon.verilization.scala_runtime

import zio.{ZIO, IO, Chunk}
import java.math.BigInteger
import scala.math.BigInt

object VLQ {


    private final case class OutputState(currentByte: Byte, outBitIndex: Int)

    def encodeVLQ[R, E](writer: FormatWriter[R, E], isSigned: Boolean, n: BigInt): ZIO[R, E, Unit] = {
        val nBytes = n.abs.toByteArray

        def putBit(b: Boolean, state: OutputState): ZIO[R, E, OutputState] =
            if(state.outBitIndex > 6) // Only use 7 bits, 8th bit is for tag to indicate more data
                writer.writeByte((state.currentByte | 0x80).toByte) *> putBit(b, OutputState(0, 0))
            else
                IO.succeed(OutputState(
                    currentByte = (state.currentByte | (1 << state.outBitIndex)).toByte,
                    outBitIndex = state.outBitIndex + 1,
                ))

        def putSign(sign: Boolean, state: OutputState): ZIO[R, E, OutputState] =
            if(state.outBitIndex != 6)
                // Pad out until the sign bit
                putBit(false, state).flatMap { state =>
                    putSign(sign, state)
                }
            else
                putBit(sign, state)
            
        def finish(state: OutputState): ZIO[R, E, Unit] =
            writer.writeByte(state.currentByte)

        def iterBits(byteIndex: Int, bitIndex: Int, zeroCount: Int, outputState: OutputState): ZIO[R, E, Unit] =
            if(byteIndex < 0) {
                if(isSigned) putSign(n < 0, outputState).flatMap(finish)
                else finish(outputState)
            }
            else {
                val bit = (nBytes(byteIndex) & (1 << bitIndex)) != 0

                val (byteIndex2, bitIndex2) =
                    if(bitIndex + 1 > 7)
                        (byteIndex - 1, 0)
                    else
                        (byteIndex, bitIndex + 1)

                def putZeroes(zeroCount: Int, outputState: OutputState): ZIO[R, E, OutputState] =
                    if(zeroCount > 0) putBit(false, outputState).flatMap { outputState => putZeroes(zeroCount - 1, outputState) }
                    else IO.succeed(outputState)

                putZeroes(
                    zeroCount = zeroCount,
                    outputState = outputState,
                ).flatMap { outputState =>
                    putBit(true, outputState)
                }.flatMap { outputState =>
                    iterBits(
                        byteIndex = byteIndex2,
                        bitIndex = bitIndex2,
                        zeroCount = if(bit) 0 else zeroCount + 1,
                        outputState = outputState,
                    )
                }
            }

        iterBits(
            byteIndex = nBytes.length - 1,
            bitIndex = 0,
            zeroCount = 0,
            outputState = OutputState(currentByte = 0, outBitIndex = 0)
        )
    }


    private final case class BigIntBuildState(currentByte: Byte, otherBytes: Chunk[Byte], bitIndex: Int) {

        def putBit(b: Boolean): BigIntBuildState = {
            val newByte = (currentByte | (1 << bitIndex)).toByte

            if(bitIndex + 1 > 7)
                BigIntBuildState(
                    currentByte = 0,
                    otherBytes = newByte +: otherBytes,
                    bitIndex = 0, 
                )
            else
                BigIntBuildState(
                    currentByte = newByte,
                    otherBytes = otherBytes,
                    bitIndex = bitIndex + 1,
                )
        }

        def toByteArray: Array[Byte] =
            (currentByte +: otherBytes).toArray

    }

    def decodeVLQ[R, E](reader: FormatReader[R, E], isSigned: Boolean): ZIO[R, E, BigInt] = {

        def processBits(state: BigIntBuildState, b: Byte, i: Int, n: Int): BigIntBuildState =
            if(i < n) processBits(state.putBit((b & (1 << i)) != 0), b, i + 1, n)
            else state

        def readBytes(state: BigIntBuildState): ZIO[R, E, BigInt] =
            reader.readByte().flatMap { b =>
                if((b & 0x80) != 0)
                    readBytes(processBits(state, b, 0, 8))
                else {
                    val sign = if((b & 0x40) != 0) -1 else 1
                    val bigInteger = new BigInteger(sign, processBits(state, b, 0, 6).toByteArray)
                    IO.succeed(BigInt(bigInteger))
                }
            }

        readBytes(BigIntBuildState(0, Chunk.empty, 0))
    }



}
