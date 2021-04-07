use num_bigint::{ BigUint, BigInt };
use num_traits::One;
use std::collections::{HashMap, HashSet};
use std::fmt;

/// A dot-separated package.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct PackageName {
	pub package: Vec<String>,
}

impl PackageName {
	pub fn from_str(pkg: &str) -> PackageName {
		PackageName {
			package: 
				if pkg.is_empty() {
					Vec::new()	
				}
				else {
					pkg.split(".").map(str::to_string).collect()
				},
		}
	}
}

impl fmt::Display for PackageName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let mut iter = self.package.iter();
		if let Some(item) = iter.next() {
			write!(f, "{}", item)?;
			while let Some(item) = iter.next() {
				write!(f, ".{}", item)?
			}
		}
		Ok(())
	}
}

/// A name that exists within a package.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct QualifiedName {
	pub package: PackageName,
	pub name: String,
}

impl fmt::Display for QualifiedName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.package.package.is_empty() {
			write!(f, "{}", self.name)
		}
		else {
			write!(f, "{}.{}", self.package, self.name)
		}
	}
}

/// A data type. This can be a built-in or user-defined type.
pub enum Type {
	Nat,
	Int,
	U8,
	I8,
	U16,
	I16,
	U32,
	I32,
	U64,
	I64,
	String,
	List(Box<Type>),
	Option(Box<Type>),
	Defined(QualifiedName),
}

/// The value of a constant.
pub enum ConstantValue {
	Integer(BigInt),
}

/// A constant definition.
pub struct Constant {
	pub value_type: Type,
	pub value: ConstantValue,
}

/// A field of a struct or enum. An enum field represents a single case.
pub struct FieldInfo {
	pub field_type: Type,
}

/// A versioned type defines the contents of a type for a specific format version.
pub struct VersionedTypeDefinition {
	pub fields: Vec<(String, FieldInfo)>,
}

/// A struct defines a product type. A struct can be defined differently in different format versions.
pub struct StructDefinition {
	pub versions: HashMap<BigUint, VersionedTypeDefinition>,
}

/// An enum defines a sum type. An enum can be defined differently in different format versions.
pub struct EnumDefinition {
	pub versions: HashMap<BigUint, VersionedTypeDefinition>,
}

/// A definition of a type. Either a struct or enum.
pub enum TypeDefinition {
	StructType(StructDefinition),
	EnumType(EnumDefinition),
}

/// Metadata about the format.
pub struct VerilizationMetadata {
	pub latest_version: BigUint,
}

/// Defines a versioned serialization format.
pub struct Verilization {
	metadata: VerilizationMetadata,
	constants: HashMap<QualifiedName, Constant>,
	type_definitions: HashMap<QualifiedName, TypeDefinition>,
}

/// Handler for iterating constant definitions.
pub trait ConstantDefinitionHandler<E> {
	/// Called for the definition of a constant.
	fn constant(&mut self, latest_version: &BigUint, name: &QualifiedName, constant: &Constant, referenced_types: HashSet<&QualifiedName>) -> Result<(), E>;
}

pub trait TypeDefinitionHandler<'model, 'state, E> {
	type StructHandlerState;
	fn begin_struct(&'state mut self, struct_name: &'model QualifiedName, referenced_types: HashSet<&'model QualifiedName>) -> Result<Self::StructHandlerState, E> where 'model : 'state;
	fn versioned_struct(state: &mut Self::StructHandlerState, explicit_version: bool, struct_name: &'model QualifiedName, version: &BigUint, type_definition: &'model VersionedTypeDefinition) -> Result<(), E> where 'model : 'state;
	fn end_struct(state: Self::StructHandlerState, struct_name: &'model QualifiedName) -> Result<(), E> where 'model : 'state;

	type EnumHandlerState;
	fn begin_enum(&'state mut self, name: &'model QualifiedName, referenced_types: HashSet<&'model QualifiedName>) -> Result<Self::EnumHandlerState, E> where 'model : 'state;
	fn versioned_enum(state: &mut Self::EnumHandlerState, explicit_version: bool, enum_name: &'model QualifiedName, version: &BigUint, type_definition: &'model VersionedTypeDefinition) -> Result<(), E> where 'model : 'state;
	fn end_enum(state: Self::EnumHandlerState, name: &'model QualifiedName) -> Result<(), E> where 'model : 'state;
}


fn find_defined_type<'a>(t: &'a Type, types: &mut HashSet<&'a QualifiedName>) {
	match t {
		Type::Defined(name) => {
			types.insert(&name);
		},
		Type::List(inner) => find_defined_type(inner, types),
		Type::Option(inner) => find_defined_type(inner, types),
		_ => (),
	}
}


fn find_referenced_types(versions: &HashMap<BigUint, VersionedTypeDefinition>) -> HashSet<&QualifiedName> {
	let mut names = HashSet::new();

	for ver_type in versions.values() {
		for (_, field) in &ver_type.fields {
			find_defined_type(&field.field_type, &mut names)
		}
	}

	names
}




impl Verilization {

	/// Creates a new versioned format.
	pub fn new(metadata: VerilizationMetadata) -> Self {
		Verilization {
			metadata: metadata,
			constants: HashMap::new(),
			type_definitions: HashMap::new(),
		}
	}

	/// Adds a constant to the serialization format.
	pub fn add_constant(&mut self, name: QualifiedName, constant: Constant) -> Result<(), QualifiedName> {
		if self.constants.contains_key(&name) {
			Err(name)
		}
		else {
			self.constants.insert(name, constant);
			Ok(())
		}
	}

	/// Adds a type definition to the serialization format.
	pub fn add_type(&mut self, name: QualifiedName, t: TypeDefinition) -> Result<(), QualifiedName> {
		if self.type_definitions.contains_key(&name) {
			Err(name)
		}
		else {
			self.type_definitions.insert(name, t);
			Ok(())
		}
	}

	pub fn has_type(&self, name: &QualifiedName) -> bool {
		self.type_definitions.contains_key(name)
	}

	pub fn has_type_in_version(&self, name: &QualifiedName, version: &BigUint) -> bool {
		match self.type_definitions.get(name) {
			Some(TypeDefinition::StructType(struct_def)) =>
				struct_def.versions.keys().find(|ver| ver <= &version).is_some(),
			Some(TypeDefinition::EnumType(enum_def)) =>
				enum_def.versions.keys().find(|ver| ver <= &version).is_some(),
			None => false,
		}
	}

	/// Merges two serialization formats.
	pub fn merge(&mut self, other: Verilization) -> Result<(), QualifiedName> {
		if self.metadata.latest_version < other.metadata.latest_version {
			self.metadata.latest_version = other.metadata.latest_version
		}

		other.constants.into_iter().try_for_each(|(name, constant)| self.add_constant(name, constant))?;
		other.type_definitions.into_iter().try_for_each(|(name, constant)| self.add_type(name, constant))?;

		Ok(())
	}


	// Iterate constants using the provided handler.
	pub fn iter_constants<E, Handler : ConstantDefinitionHandler<E>>(&self, handler: &mut Handler) -> Result<(), E> {
		for (name, constant) in &self.constants {
			let mut referenced_types = HashSet::new();
			find_defined_type(&constant.value_type, &mut referenced_types);
			handler.constant(&self.metadata.latest_version, &name, &constant, referenced_types)?
		}

		Ok(())
	}

	// Iterates type definitions using the provided handler.
	pub fn iter_types<'model, E, Handler : for<'state> TypeDefinitionHandler<'model, 'state, E>>(&'model self, handler: &mut Handler) -> Result<(), E> {
		let latest_version = &self.metadata.latest_version;

		for (name, t) in &self.type_definitions {
			match t {
				TypeDefinition::StructType(struct_def) => {
					let referenced_types = find_referenced_types(&struct_def.versions);
					let mut state = handler.begin_struct(&name, referenced_types)?;
					
					let mut version: BigUint = One::one();
					let mut last_seen_version = None;
					while version <= *latest_version {
						
						if let Some(ver_struct) = struct_def.versions.get(&version) {
							Handler::versioned_struct(&mut state, true, &name, &version, &ver_struct)?;
							last_seen_version = Some(ver_struct);
						}
						else if let Some(ver_struct) = last_seen_version {
							Handler::versioned_struct(&mut state, false, &name, &version, &ver_struct)?;
						}
						
						version = version + <BigUint as One>::one();
					}

					Handler::end_struct(state, &name)?
				},

				TypeDefinition::EnumType(enum_def) => {
					let referenced_types = find_referenced_types(&enum_def.versions);
					let mut state = handler.begin_enum(&name, referenced_types)?;
					
					let mut version: BigUint = One::one();
					let mut last_seen_version = None;
					while version <= *latest_version {
						
						if let Some(ver_enum) = enum_def.versions.get(&version) {
							Handler::versioned_enum(&mut state, true, &name, &version, &ver_enum)?;
							last_seen_version = Some(ver_enum);
						}
						else if let Some(ver_enum) = last_seen_version {
							Handler::versioned_enum(&mut state, false, &name, &version, &ver_enum)?;
						}
						
						version = version + <BigUint as One>::one();
					}

					Handler::end_enum(state, &name)?
				},
			}
		}

		Ok(())
	}

}


