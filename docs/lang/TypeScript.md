# Verilization for TypeScript

## Options

 * `out_dir` - the base output directory
 * `pkg:package.name` - the subdirectory for the package
 * `lib:package.name` - the module import for the library, types in this package will not be generated


## Runtime

| Verilization type | TypeScript type |
|---|---|
| {`i`,`u`}{`8`,`16`,`32`} | `number` |
| {`i`,`u`}`64`, `int`, and `nat` | `bigint` |
| `string` | `string` |
| `list(T)` | A custom type that is the union of a `readonly T[]` and any applicable typed arrays |
| `option(T)` | `{ readonly value: T } | null` |

## Generation

Each type will generate a separate module.
A module will export types of the form `Vn` where `n` is the version, as well as a namespace of the same name.
The namespace will export the following functions and values.

 * `converter` - Generated for the last version of `final` types that have type parameters. Accepts arguments for converters of each type parameter.
 * `fromVn` - Generated for all but the first version of types. Accepts arguments for converters of each type parameter. Generated as a `const` for types without type parameters.
 * `codec` - Accepts arguments for codecs of each type parameter. Generated as a `const` for types without type parameters.

The capitalization of identifiers will be adjusted. For example:

 * Field names will have the first character converted to lower case.
 * `case` constructor functions will convert the first character to upper case.
 * The file name of the types will convert the first character to upper case.

## Defining `extern` types

 * An `extern` type must define a module in the location where the file would have been generated for a versioned type.
 * The module must export a type with the same name as the file.
 * The module must export a codec in the same manner as a generated module.
 * If the type is parameterized, the module must export a converter in the same manner as a generated module.
 * If the type defines literals, functions must be exported as shown in the table below.

| Literal | Export Signature |
|---|---|
| `integer ...` | `function fromInteger(n: bigint): X` |
| `string` | `function fromString(s: string): X` |
| `sequence T` | `function fromSequence(...seq: T[]): X` |
| `case MyCase(T1, ...)` | `function fromCaseMyCase(t1: T1, ...): X` |
| `record { field1: T1, ... }` | `function fromRecord(value: { field1: T1, ... }): X` <br /> `fromRecord` is called using named arguments. The names of the parameters must match the field names in the record. |