use num_bigint::{ BigUint, BigInt };
use num_traits::One;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::FromIterator;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};

/// A dot-separated package.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct PackageName {
	pub package: Vec<String>,
}

impl PackageName {
	pub fn new() -> PackageName {
		PackageName {
			package: Vec::new(),
		}
	}
	
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

	pub fn from_parts(parts: &[&str]) -> PackageName {
		PackageName {
			package: Vec::from_iter(parts.iter().map(|x| x.to_string())),
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

impl Ord for PackageName {
	fn cmp(&self, other: &Self) -> Ordering {
		let mut i1 = self.package.iter();
		let mut i2 = other.package.iter();

		loop {
			match (i1.next(), i2.next()) {
				(Some(p1), Some(p2)) => {
					let ord = p1.cmp(p2);
					if ord != Ordering::Equal {
						return ord
					}
				},
				(Some(_), None) => return Ordering::Greater,
				(None, Some(_)) => return Ordering::Less,
				(None, None) => return Ordering::Equal,
			}
		}
	}
}

impl PartialOrd for PackageName {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
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

impl Ord for QualifiedName {
	fn cmp(&self, other: &Self) -> Ordering {
		let ord = self.package.cmp(&other.package);
		if ord != Ordering::Equal {
			return ord
		}

		self.name.cmp(&other.name)
	}
}

impl PartialOrd for QualifiedName {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

/// A data type. This can be a built-in or user-defined type.
#[derive(Clone)]
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
	Defined(QualifiedName, Vec<Type>),
}

/// The value of a constant.
pub enum ConstantValue {
	Integer(BigInt),
}

/// A constant definition.
pub struct Constant {
	pub imports: HashMap<String, ScopeLookup>,
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
pub struct TypeDefinitionData {
	pub imports: HashMap<String, ScopeLookup>,
	pub type_params: Vec<String>,
	pub versions: HashMap<BigUint, VersionedTypeDefinition>,
}

/// An enum defines a sum type. An enum can be defined differently in different format versions.
pub struct EnumDefinition {
	pub versions: HashMap<BigUint, VersionedTypeDefinition>,
}

/// A definition of a type. Either a struct or enum.
pub enum TypeDefinition {
	StructType(TypeDefinitionData),
	EnumType(TypeDefinitionData),
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

#[derive(Clone)]
pub enum ScopeLookup {
	NamedType(QualifiedName),
	TypeParameter(String),
}

pub struct Scope<'a> {
	model: &'a Verilization,
	current_pkg: &'a PackageName,
	imports: &'a HashMap<String, ScopeLookup>,
	type_params: &'a Vec<String>,
}

impl <'a> Scope<'a> {
	pub fn lookup(&self, mut name: QualifiedName) -> ScopeLookup {
		if name.package.package.is_empty() {
			if self.type_params.contains(&name.name) {
				return ScopeLookup::TypeParameter(name.name);
			}

			if let Some(import) = self.imports.get(&name.name) {
				return import.clone();
			}

			let current_pkg_name = QualifiedName {
				package: self.current_pkg.clone(),
				name: name.name,
			};

			if self.model.has_type(&current_pkg_name) {
				return ScopeLookup::NamedType(current_pkg_name);
			}

			name.name = current_pkg_name.name; // restore name because current_pkg_name is not a type
		}

		ScopeLookup::NamedType(name)
	}
}

/// Handler for iterating constant definitions.
pub trait ConstantDefinitionHandler<E> {
	/// Called for the definition of a constant.
	fn constant(&mut self, latest_version: &BigUint, name: &QualifiedName, scope: &Scope, constant: &Constant, referenced_types: HashSet<&QualifiedName>) -> Result<(), E>;
}

pub trait TypeDefinitionHandlerState<'model, 'state, 'scope, Outer, E> : Sized where Outer : TypeDefinitionHandler<'model, E>, 'model : 'state {
	fn begin(outer: &'state mut Outer, type_name: &'model QualifiedName, type_params: &'model Vec<String>, scope: &'scope Scope<'model>, referenced_types: HashSet<&'model QualifiedName>) -> Result<Self, E>;
	fn versioned_type(&mut self, explicit_version: bool, version: &BigUint, type_definition: &'model VersionedTypeDefinition) -> Result<(), E>;
	fn end(self) -> Result<(), E>;
}

pub trait TypeDefinitionHandler<'model, E> : Sized {
	type StructHandlerState<'state, 'scope> : TypeDefinitionHandlerState<'model, 'state, 'scope, Self, E> where 'model : 'scope, 'scope : 'state;
	type EnumHandlerState<'state, 'scope> : TypeDefinitionHandlerState<'model, 'state, 'scope, Self, E> where 'model : 'scope, 'scope : 'state;
}

trait HandlerStateSelector<'model, E, Outer : TypeDefinitionHandler<'model, E>> {
	type HandlerState<'state, 'scope> : TypeDefinitionHandlerState<'model, 'state, 'scope, Outer, E> where 'model : 'scope, 'scope : 'state;
}

struct StructSelector {}
impl <'model, E, Handler : TypeDefinitionHandler<'model, E>> HandlerStateSelector<'model, E, Handler> for StructSelector {
	type HandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = Handler::StructHandlerState<'state, 'scope>;
}

struct EnumSelector {}
impl <'model, E, Handler : TypeDefinitionHandler<'model, E>> HandlerStateSelector<'model, E, Handler> for EnumSelector {
	type HandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = Handler::EnumHandlerState<'state, 'scope>;
}

fn find_defined_type<'a>(t: &'a Type, types: &mut HashSet<&'a QualifiedName>) {
	match t {
		Type::Defined(name, args) => {
			types.insert(&name);
			for arg in args {
				find_defined_type(arg, types);
			}
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

impl TypeDefinition {
	pub fn arity(&self) -> usize {
		match self {
			TypeDefinition::StructType(t) => t.type_params.len(),
			TypeDefinition::EnumType(t) => t.type_params.len(),
		}
	}

	pub fn has_version(&self, version: &BigUint) -> bool {
		match self {
			TypeDefinition::StructType(t) => t.versioned(version).is_some(),
			TypeDefinition::EnumType(t) => t.versioned(version).is_some(),
		}
	}
}

impl TypeDefinitionData {
	pub fn versioned<'a>(&'a self, version: &BigUint) -> Option<&'a VersionedTypeDefinition> {
		let (_, ver_type) = self.versions.iter()
			.filter(|(ver, _)| ver <= &version)
			.max_by_key(|(ver, _)| ver.clone())?;

		Some(ver_type)
	}
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

	pub fn get_type<'a>(&'a self, name: &QualifiedName) -> Option<&'a TypeDefinition> {
		self.type_definitions.get(name)
	}

	pub fn has_type<'a>(&'a self, name: &QualifiedName) -> bool {
		self.type_definitions.contains_key(name)
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
			let type_params = Vec::new();
			let scope = Scope {
				model: self,
				current_pkg: &name.package,
				imports: &constant.imports,
				type_params: &type_params,
			};

			let mut referenced_types = HashSet::new();
			find_defined_type(&constant.value_type, &mut referenced_types);
			handler.constant(&self.metadata.latest_version, &name, &scope, &constant, referenced_types)?
		}

		Ok(())
	}

	// Iterates type definitions using the provided handler.
	pub fn iter_types<'model, E, Handler : TypeDefinitionHandler<'model, E>>(&'model self, handler: &mut Handler) -> Result<(), E> {
		let latest_version = &self.metadata.latest_version;

		for (name, t) in &self.type_definitions {

			fn handle_type<'model, 'state, E, Handler : TypeDefinitionHandler<'model, E>, Selector : HandlerStateSelector<'model, E, Handler>>(handler: &'state mut Handler, model: &'model Verilization, latest_version: &BigUint, name: &'model QualifiedName, type_def: &'model TypeDefinitionData) -> Result<(), E> where 'model : 'state {
				let scope = Scope {
					model: model,
					current_pkg: &name.package,
					imports: &type_def.imports,
					type_params: &type_def.type_params,
				};
				
				let referenced_types = find_referenced_types(&type_def.versions);
				let mut state = Selector::HandlerState::<'_, '_>::begin(handler, name, &type_def.type_params, &scope, referenced_types)?;
				
				let mut version = BigUint::one();
				let mut last_seen_version = None;
				while version <= *latest_version {
					
					if let Some(ver_struct) = type_def.versions.get(&version) {
						state.versioned_type(true, &version, &ver_struct)?;
						last_seen_version = Some(ver_struct);
					}
					else if let Some(ver_struct) = last_seen_version {
						state.versioned_type(false, &version, &ver_struct)?;
					}
					
					version = version + BigUint::one();
				}

				state.end()?;

				Ok(())
			}


			match t {
				TypeDefinition::StructType(struct_def) => handle_type::<_, _, StructSelector>(handler, self, latest_version, &name, &struct_def)?,
				TypeDefinition::EnumType(enum_def) => handle_type::<_, _, EnumSelector>(handler, self, latest_version, &name, &enum_def)?,
			}
		}

		Ok(())
	}

}


