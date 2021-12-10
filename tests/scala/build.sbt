
lazy val runtimeLib = ProjectRef(file("../../runtime/scala"), "scalaRuntimeJVM")

lazy val proj = crossProject(JSPlatform, JVMPlatform).in(file("."))
    .jvmConfigure(_.dependsOn(ProjectRef(file("../../runtime/scala"), "scalaRuntimeJVM")))
    .jsConfigure(_.dependsOn(ProjectRef(file("../../runtime/scala"), "scalaRuntimeJS")))
    .settings(
        name := "verilization-tests",
        organization := "dev.argon",
        crossPaths := false,
        Compile / unmanagedSourceDirectories += baseDirectory.value / "../gen",
        Test / unmanagedSourceDirectories += baseDirectory.value / "../gen-test",

        scalacOptions ++= Seq(
            "-deprecation",
        ),


        crossScalaVersions := List("3.1.0", "2.13.7", "2.12.15"),
        libraryDependencies ++= Seq(
            "dev.zio" %% "zio" % "2.0.0-M6-2",
            "dev.zio" %% "zio-test" % "2.0.0-M6-2" % Test,
            "dev.zio" %% "zio-test-sbt" % "2.0.0-M6-2" % Test,
        ),

        testFrameworks += new TestFramework("zio.test.sbt.ZTestFramework"),
    )

lazy val projJVM = proj.jvm
lazy val projJS = proj.js
