# Verilization for Java

## Options

 * `out_dir` - the base output directory
 * `pkg:package.name` - the Java package mapping for the package
 * `lib:package.name` - the Java package mapping for the library package, types in this package will not be generated
 * `extern:type.name` - the Java type mapping for the given extern type (optional)


## Runtime

| Verilization type | Java type |
|---|---|
| {`i`,`u`}`8` | `byte` |
| {`i`,`u`}`16`} | `short` |
| {`i`,`u`}`32` | `int` |
| {`i`,`u`}`64` | `long` |
| `int` and `nat` | `BigInteger` |
| `string` | `string` |
| `list(T)` | A custom type that is immutable and can contain unboxed values of primitive types |
| `option(T)` | `Optional<T>` |

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

 * An `extern` type must define a class with the same name as the type that would have been generated for a versioned type.
    * This applies even when the type is mapped. However, in this case the class will not be used as the type. It will only be used for the methods.
 * The class must define a codec in the same manner as a generated module.
 * If the type is parameterized, the class must define a converter in the same manner as a generated module.
 * If the type defines literals, the class must define static methods as shown in the table below.

| Literal | Method Signature |
|---|---|
| `integer ...` | `X fromInteger(int i)` <br /> `X fromInteger(long l)` <br /> `X fromInteger(BigInteger i)` <br /> If the value is in range of an `int` or `long`, those overloads will be used. |
| `string` | `X fromString(String s)` |
| `sequence T` | `X fromSequence(T... seq)` |
| `case MyCase(T1, ...)` | `X fromCaseMyCase(t1: T1, ...)` |
| `record { field1: T1, ... }` | `X fromRecord(field1: T1, ...)` |