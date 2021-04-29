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

## Types

The following types are supported.

|Type|Encoding|
|---|---|
| `struct` types | The encoding of each field in order |
| `enum` types | A tag (encoded in the same format as `nat`) followed by the encoding of the field represented by the tag |
| `extern` types | Defined by code written in the target language |

### Structs

A `struct` type is defined with multiple [versions](#versioning). Each version defines a list of fields.

    struct Rectangle {
        version 1 {
            width: u32;
            height: u32;
        }
    }

### Enums

An `enum` type is defined with multiple [versions](#versioning). Each version defines a list of fields used as cases. An enum value consists of exactly one of these fields.

    struct StringOrInt {
        version 1 {
            str: string;
            num: int;
        }
    }

### Externs

An `extern` type is defined in user code. The type definition, conversions, and codecs must be implemented in the target language.

An `extern` type may declare what literals may be used for the type.

    extern MyString {
        literal {
            string;
        }
    }

The following literal specifications are supported.

| Name | Example | Syntax | Notes |
|---|---|---|---|
| Integer | integer [0, 256) | `'integer' open_bracket integer_literal? ',' integer_literal? close_bracket` <br /> where `open_bracket : '[' | '('` and `close_bracket : ']' | ')'` | Defines the range of allowed integers. Square brackets are inclusive, parentheses are exclusive. Omit the number for an infinite range.
| String | `string` | `'string'` | The contents of the string cannot be restricted. |
| Sequence | `sequence T` | `'sequence' type_expr` | Defines a sequence of the specified type. |
| Case | `case Positive()` | `'case' identifier '(' [ type_expr { ',' type_expr } ] ')'` | Defines a case. Multiple case literals may be specified if the names are distinct. |
| Record| `record { a: A; b: B; }` | `'record' '{' { identifier ':' type_expr ';' } '}'` | Defines a record. |

 * `integer` - 

### Runtime Library Types

There are a number of `extern` types provided by the runtime library.

|Type|Literals|Encoding|
|---|---|---|
| `{i,u}{8,16,32,64}` | Integers within the range of the type | Fixed-width sequence of bytes in little endian order |
| `int` | Integers | A variable-length format |
| `nat` | Non-negative integers | Similar format to `int`, but without the sign bit |
| `string` | Strings | A length `nat` followed by a sequence of UTF-8 bytes with the specified length |
| `list T` | sequence of `T` | A length `nat` followed by a sequence of `T` |
| `option T` | Two cases `some(x)` and `none()` | A byte `b`. If `b` is non-zero, then it is followed by a `T` |

The encodings for `int` and `nat` define a sequence of bits in little-endian order.
The highest bit in each byte is set if there are more bytes in the number.

This encoding is a sequence of bytes [B<sub>0</sub>, ..., B<sub>n</sub>] where B<sub>i,7</sub> = 1 when i < n and B<sub>n,7</sub> = 0.
This sequence of bytes is equivalent to a sequence of bits [B<sub>0,0</sub>, ... B<sub>0,6</sub>, ..., B<sub>n-1,0</sub>, ..., B<sub>n-1,6</sub>] = [b<sub>0</sub>, ..., b<sub>m-1</sub>] where m = 6n.
Essentially, the sequence of bits removes the flag bits that are used to determine when the sequence has reached the end and orders the remaining bits in each byte from least to most significant.
The sequence of bits is mapped as follows:

 * For the `int` type, if b<sub>m-1</sub> = 0, then k = b<sub>0</sub> * 2<sup>0</sup> + ... + b<sub>m - 2</sub> * 2<sup>m-2</sup>
 * For the `int` type, if b<sub>m-1</sub> = 1, then k = -(b<sub>0</sub> * 2<sup>0</sup> + ... + b<sub>m - 2</sub> * 2<sup>m-2</sup>) - 1
 * For the `nat` type, k = b<sub>0</sub> * 2<sup>0</sup> + ... + b<sub>m - 1</sub> * 2<sup>m-1</sup>

## <a name="versioning">Versioning</a>

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

## Generics

Generic types allow a type to be parameterized.

    final struct Pair(A, B) {
        version 1 {
            left: A;
            right: B;
        }
    }

## Constants

Constants allow for values to be defined that are shared between any generated languages.

| Literal | Example | Usage |
|---|---|---|
| Integer | `88` | `extern` types with `integer` literal |
| String| `"Hello World"` | `extern` types with `string` literal |
| Sequence | `[ a, b, c ]` | `extern` types with `sequence` literal |
| Record | `{ x = 1; y = 2; }` | `struct` types and `extern` types with `record` literal |
| Case | `Name(a)` | `enum` types and `extern` types with `case Name` literal |

## Command Line

Verilization has a command line interface. The following options are supported.

## Language Generators

The following languages are supported.

 * [TypeScript](docs/lang/TypeScript.md)
 * [Java](docs/lang/Java.md)
 * [Scala](docs/lang/Scala.md)

## Compiler Bindings

The verilization compiler is written in Rust.
It can be compiled into WebAssembly for use in other languages.
This has the advantage that a tool can be distributed (for example, as an NPM package, a standalone JAR, etc) without requiring any native binaries.
These bindings expose both an interface that can be used directly from the runtime, as well as a command line interface that depends only on the associated runtime.

Currently, there are bindings for the following runtimes.

 * Node

