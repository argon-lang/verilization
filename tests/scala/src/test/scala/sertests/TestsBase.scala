package sertests

import java.io.IOException
import zio.test._
import zio.test.Assertion._
import zio.IO

abstract class TestsBase extends DefaultRunnableSpec {

    protected def testCases: Seq[TestCase[_]]

    private def encodeTest[A](testCase: TestCase[A]): IO[IOException, TestResult] =
        for {
            writer <- MemoryFormatWriter.make
            _ <- testCase.codec.write(writer, testCase.value)
            actual <- writer.toChunk
        } yield assert(actual)(equalTo(testCase.encoded))

    private def decodeTest[A](testCase: TestCase[A]): IO[IOException, TestResult] =
        for {
            reader <- MemoryFormatReader.fromChunk(testCase.encoded)
            actual <- testCase.codec.read(reader)
            eof <- reader.isEOF
        } yield assert(eof)(isTrue) && assert(actual)(equalTo(testCase.value))


    override def spec: ZSpec[Environment, Failure] =
        suite("Verilization tests")(testCases.zipWithIndex.flatMap { case (testCase, i) =>
            Seq(
                testM(s"test $i encode")(encodeTest(testCase)),
                testM(s"test $i decode")(decodeTest(testCase)),
            )
        }: _*)
}

