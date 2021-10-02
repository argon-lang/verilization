
lazy val runtimeLib = RootProject(file("../../runtime/java"))

lazy val proj = project.in(file("."))
    .dependsOn(runtimeLib)
    .settings(
        name := "verilization-tests",
        organization := "dev.argon",
        crossPaths := false,
        autoScalaLibrary := false,
        unmanagedSourceDirectories in Compile += baseDirectory.value / "gen",
        unmanagedSourceDirectories in Test += baseDirectory.value / "gen-test",
	
        libraryDependencies ++= Seq(
            "net.aichler" % "jupiter-interface" % JupiterKeys.jupiterVersion.value % Test,
        ),

        javacOptions ++= Seq(
            "--release", "17",
            "--enable-preview",
            "-encoding", "UTF-8",
        ),
    )
