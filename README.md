# Verilization

Verilization is a serilization description language designed for defining binary file formats.
Unlike other serialization tools such as Protocol Buffers, serialized Verilization data is not forward or backward compatible.
Instead, conversions from older format versions are made easy, allowing for more compact data and more control over the underlying shape of the data.

## Goals

Verilization has the following primary goals.

 * Give maximum control of the file format to the user
 * Define the format in a language-independent manner
 * Support easy conversions from older versions of the format

Other less high-level goals.

 * Support bigint types
 * Embedable in other languages without using native binaries

## Versioning

In the following example, a user has a username and birth date.

    struct Person {
        version 1 {
            name: Name;
            dob: Date;
        }
    }

    struct Name {
        version 1 {
            firstName: string;
            middleName: option string;
            lastName: string;
        }
    }

However, not everyone has 2 or 3 names. In order to accomodate this, we can create a new version that allows for an arbitrary number of names.

    struct Name {
        version 1 {
            ...
        }
        version 2 {
            names: list string;
        }
    }

This change to `Name` means that in version 2 of the format, the `name` field of `Person` will now use version 2 of `Name`.
However, since there are no direct changes to `Person`, version 2 is automaticly created.
In the generated code, the user is expected to provide code that can upgrade `Name` from version 1 to version 2.
However, there is no need to provide such code for upgrading `Person`.
`Person` can be upgraded automaticially using the upgrade code for its fields.


## Command Line

Verilization has a command line interface. The following options are supported.

## Language Generators

The following languages are supported.

### TypeScript

 * small integer types ({i,u}{8,16,32}) are mapped to number
 * other integer types ({i,u}64, int, and nat) are mapped to bigint

#### TypeScript-specific Options

 * `out_dir` - the base output directory
 * `pkg:package.name` - the subdirectory for the package

### Java

 * Unsinged integer types are mapped to their signed equivalent

#### Java-specific Options

 * `out_dir` - the base output directory
 * `pkg:package.name` - the java package for the original verilization package

### Scala

 * Unsinged integer types are mapped to their signed equivalent

#### Scala-specific Options

 * `out_dir` - the base output directory
 * `pkg:package.name` - the scala package for the original verilization package

## Compiler Bindings

The verilization compiler is written in Rust.
It can be compiled into WebAssembly for use in other languages.
This has the advantage that a tool can be distributed (for example, as an NPM package, a standalone JAR, etc) without requiring any native binaries.
These bindings expose both an interface that can be used directly from the runtime, as well as a command line interface that depends only on the associated runtime.

Currently, there are bindings for the following runtimes.

 * Node


