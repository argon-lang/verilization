# Verilization for Scala

## Options

 * `out_dir` - the base output directory
 * `pkg:package.name` - the Scala package mapping for the package
 * `lib:package.name` - the Scala package mapping for the library package, types in this package will not be generated


## Runtime

| Verilization type | Scala type |
|---|---|
| {`i`,`u`}`8` | `Byte` |
| {`i`,`u`}`16`} | `Short` |
| {`i`,`u`}`32` | `Int` |
| {`i`,`u`}`64` | `Long` |
| `int` and `nat` | `BigInt` |
| `string` | `String` |
| `list(T)` | `zio.Chunk[T]` |
| `option(T)` | `Option[T]` |

## Generation

Each type will generate a separate top-level class.
This class will expose public types of the form `Vn` where `n` is the version, as well as a namespace of the same name.
Each version type have the following public functions and values.

 * `converter` - Generated for the last version of `final` types that have type parameters. Accepts arguments for converters of each type parameter.
 * `fromVn` - Generated for all but the first version of types. Accepts arguments for converters of each type parameter. Generated as a `const` for types without type parameters.
 * `codec` - Accepts arguments for codecs of each type parameter. Generated as a `const` for types without type parameters.

The capitalization of identifiers will be adjusted. For example:

 * Field names will have the first character converted to lower case.
 * `case` constructor functions will convert the first character to upper case.
 * Type names will convert the first character to upper case.

## Defining `extern` types

 * An `extern` type must define a type with the same name as the type that would have been generated for a versioned type.
 * A corresponding object must be declared.
 * The object must define a codec in the same manner as a generated module.
 * If the type is parameterized, the object must define a converter in the same manner as a generated module.
 * If the type defines literals, the object must define methods methods as shown in the table below.

| Literal | Method Signature |
|---|---|
| `integer ...` | `def fromInteger(i: Int): X` <br /> `def fromInteger(l: Long): X` <br /> `def fromInteger(i: BigInt): X` <br /> If the value is in range of an `int` or `long`, those overloads will be used. |
| `string` | `def fromString(s: String): X` |
| `sequence T` | `def fromSequence(seq: T*): X` |
| `case MyCase(T1, ...)` | `def fromCaseMyCase(t1: T1, ...): X` |
| `record { field1: T1, ... }` | `def fromRecord(field1: T1, ... ): X` |