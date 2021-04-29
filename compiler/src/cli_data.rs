
pub const HELP_MESSAGE: &str = "

verilization <command> [<args>]

Commands:

    version                      Displays the version of the verilization compiler.
    help                         Displays this message.
    generate <lang> [<args>]     Generates serilization code for a given language.

        Supported Languages:
            typescript
            java
            scala

        Common Options:
            -i                   Adds an input source file.

        TypeScript specific options:
            -o:out_dir           The output directory.
            -o:pkg:<package>     The subdirectory where types defined in the package will be placed.
            -o:lib:<package>     The module import for the specified package. Types in this package will not be generated.

        Java specific options:
            -o:out_dir           The output directory.
            -o:pkg:<package>     The Java package where types defined in the package will be placed.
            -o:lib:<package>     The Java package for the specified package. Types in this package will not be generated.
            -o:extern:<type>     The Java type that will be used as the actual data type.

            Scala specific options:
            -o:out_dir           The output directory.
            -o:pkg:<package>     The Scala package where types defined in the package will be placed.
            -o:lib:<package>     The Scala package for the specified package. Types in this package will not be generated.

";
