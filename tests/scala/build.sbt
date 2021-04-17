
lazy val runtimeLib = RootProject(file("../../runtime/scala"))

lazy val proj = project.in(file("."))
    .dependsOn(runtimeLib)
    .settings(
        name := "verilization-tests",
        organization := "dev.argon",
        crossPaths := false,
        Compile / unmanagedSourceDirectories += baseDirectory.value / "gen",
        Test / unmanagedSourceDirectories += baseDirectory.value / "gen-test",


        scalaVersion := "2.13.5",
        libraryDependencies ++= Seq(
            "dev.zio" %% "zio" % "1.0.5",
            "dev.zio" %% "zio-test" % "1.0.5" % Test,
            "dev.zio" %% "zio-test-sbt" % "1.0.5" % Test,
        ),

        testFrameworks += new TestFramework("zio.test.sbt.ZTestFramework"),
    )
