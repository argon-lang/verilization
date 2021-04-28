use num_bigint::{ BigUint, BigInt };
use num_traits::{Zero, One};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::iter::FromIterator;
use std::cmp::{Ord, Ordering, PartialEq, PartialOrd};
use lazy_static::lazy_static;
use std::marker::PhantomData;

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
			package: parts.iter().map(|x| String::from(*x)).collect::<Vec<_>>(),
		}
	}
}

impl fmt::Display for PackageName {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		for_sep!(item, &self.package, { write!(f, ".")?; }, {
			write!(f, "{}", item)?;
		});
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

impl QualifiedName {
	pub fn from_str(name: &str) -> Option<QualifiedName> {
		let mut iter = name.split(".");
		if let Some(part) = iter.next() {
			let mut pkg_parts = Vec::new();
			let mut name = String::from(part);

			while let Some(part) = iter.next() {
				pkg_parts.push(name);
				name = String::from(part);
			}

			Some(QualifiedName {
				package: PackageName {
					package: pkg_parts,
				},
				name: name,
			})
		}
		else {
			None
		}
	}

	pub fn from_parts(package: &[&str], name: &str) -> QualifiedName {
		QualifiedName {
			package: PackageName::from_parts(package),
			name: String::from(name),
		}
	}
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
#[derive(Clone, Debug)]
pub enum Type {
	Defined(QualifiedName, Vec<Type>),
}

// Attaches a name to something.
pub struct Named<'a, A> {
	model: &'a Verilization,
	name: &'a QualifiedName,
	value: &'a A,
}

impl <'a, A: fmt::Debug> fmt::Debug for Named<'a, A> {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		f.debug_struct("Named")
			.field("name", &self.name)
			.field("value", &self.value)
			.finish()
	}
}

impl <'a, A> Clone for Named<'a, A> {
	fn clone(&self) -> Self {
		Named {
			model: self.model,
			name: self.name,
			value: self.value,
		}
	}
}

impl <'a, A> Copy for Named<'a, A> {}

impl <'a, A> Named<'a, A> {
	fn new(model: &'a Verilization, name: &'a QualifiedName, value: &'a A) -> Named<'a, A> {
		Named {
			model: model,
			name: name,
			value: value,
		}
	}

	pub fn name(&self) -> &'a QualifiedName {
		self.name
	}
}

/// The value of a constant.
pub enum ConstantValue {
	Integer(BigInt),
	String(String),
	Sequence(Vec<ConstantValue>),
	Case(String, Vec<ConstantValue>),
	Record(HashMap<String, ConstantValue>),
	Constant(QualifiedName),
}

pub struct ConstantVersionInfo<'a> {
	pub version: BigUint,
	pub value: Option<&'a ConstantValue>,

	dummy: PhantomData<()>,
}

/// A constant definition.
pub struct Constant {
	pub(crate) imports: HashMap<String, ScopeLookup>,
	pub(crate) value_type: Type,
	pub(crate) versions: HashMap<BigUint, ConstantValue>,
}

impl <'a> Named<'a, Constant> {

	pub fn value_type(self) -> &'a Type {
		&self.value.value_type
	}

	pub fn referenced_types(self) -> ReferencedTypeIterator<'a> {
		ReferencedTypeIterator::from_type(&self.value.value_type)
	}

	pub fn versions(self) -> ConstantVersionIterator<'a> {
		ConstantVersionIterator {
			constant: self,
			version: BigUint::one(),
			has_prev_version: false,
			max_version: self.model.metadata.latest_version.clone(),
		}
	}

	pub fn scope(self) -> Scope<'a> {
		Scope {
			model: self.model,
			current_pkg: Some(&self.name.package),
			imports: Some(&self.value.imports),
			type_params: None,
		}
	}

}

/// A field of a struct or enum. An enum field represents a single case.
#[derive(Debug)]
pub struct FieldInfo {
	pub field_type: Type,

	pub(crate) dummy: PhantomData<()>,
}

/// A versioned type defines the contents of a type for a specific format version.
#[derive(Debug)]
pub struct TypeVersionDefinition {
	pub fields: Vec<(String, FieldInfo)>,

	pub(crate) dummy: PhantomData<()>,
}

#[derive(Debug)]
pub struct TypeVersionInfo<'a> {
	pub version: BigUint,
	pub explicit_version: bool,
	pub ver_type: &'a TypeVersionDefinition,

	dummy: PhantomData<()>,
}

impl <'a> Clone for TypeVersionInfo<'a> {
	fn clone(&self) -> Self {
		TypeVersionInfo {
			version: self.version.clone(),
			explicit_version: self.explicit_version,
			ver_type: self.ver_type,

			dummy: PhantomData {}
		}
	}
}

/// A struct defines a product type. A struct can be defined differently in different format versions.
#[derive(Debug)]
pub struct VersionedTypeDefinitionData {
	pub(crate) imports: HashMap<String, ScopeLookup>,
	pub(crate) type_params: Vec<String>,
	pub(crate) versions: HashMap<BigUint, TypeVersionDefinition>,
	pub(crate) is_final: bool,
}

impl <'a> Named<'a, VersionedTypeDefinitionData> {
	pub fn is_final(&self) -> bool {
		self.value.is_final
	}

	pub fn versioned(self, version: &BigUint) -> Option<TypeVersionInfo<'a>> {
		self.value.versions.iter()
			.filter(|(ver, _)| ver <= &version)
			.max_by_key(|(ver, _)| ver.clone())
			.map(|(actual_ver, ver_type)| {

				let ver =
					if self.value.is_final && !self.value.versions.keys().any(|other_ver| other_ver > actual_ver) {
						actual_ver.clone()
					}
					else {
						version.clone()
					};

				TypeVersionInfo {
					version: ver,
					explicit_version: version == actual_ver,
					ver_type: ver_type,
					dummy: PhantomData {},
				}
			})
	}

	pub fn last_explicit_version(self) -> Option<&'a BigUint> {
		self.value.versions.keys().max()
	}

	pub fn referenced_types(self) -> ReferencedTypeIterator<'a> {
		ReferencedTypeIterator::from_versions(&self.value.versions)
	}

	pub fn versions(self) -> TypeVersionIterator<'a> {
		TypeVersionIterator {
			type_def: self,
			version: BigUint::one(),
			max_version:
				if self.value.is_final {
					self.last_explicit_version().map(|ver| ver.clone()).unwrap_or(BigUint::zero())
				}
				else {
					self.model.metadata.latest_version.clone()
				},

			last_seen_version: None,
		}
	}

	pub fn scope(self) -> Scope<'a> {
		Scope {
			model: self.model,
			current_pkg: Some(&self.name.package),
			imports: Some(&self.value.imports),
			type_params: Some(&self.value.type_params),
		}
	}

	pub fn type_params(self) -> &'a Vec<String> {
		&self.value.type_params
	}
}

#[derive(Debug)]
pub struct ExternTypeDefinitionData {
	pub(crate) imports: HashMap<String, ScopeLookup>,
	pub(crate) type_params: Vec<String>,
	pub(crate) literals: Vec<ExternLiteralSpecifier>,
}

#[derive(Debug)]
pub enum ExternLiteralIntBound {
	Inclusive,
	Exclusive,
}

#[derive(Debug)]
pub enum ExternLiteralSpecifier {
	Integer(ExternLiteralIntBound, Option<BigInt>, ExternLiteralIntBound, Option<BigInt>),
	String,
	Sequence(Type),
	Case(String, Vec<Type>),
	Record(Vec<(String, FieldInfo)>),
}

impl <'a> Named<'a, ExternTypeDefinitionData> {
	pub fn literals(self) -> &'a Vec<ExternLiteralSpecifier> {
		&self.value.literals
	}

	pub fn scope(self) -> Scope<'a> {
		Scope {
			model: self.model,
			current_pkg: Some(&self.name.package),
			imports: Some(&self.value.imports),
			type_params: Some(&self.value.type_params),
		}
	}

	pub fn type_params(self) -> &'a Vec<String> {
		&self.value.type_params
	}
}



/// A definition of a type. Either a struct or enum.
pub enum TypeDefinition {
	StructType(VersionedTypeDefinitionData),
	EnumType(VersionedTypeDefinitionData),
	ExternType(ExternTypeDefinitionData),
}

#[derive(Copy, Clone)]
pub enum NamedTypeDefinition<'a> {
	StructType(Named<'a, VersionedTypeDefinitionData>),
	EnumType(Named<'a, VersionedTypeDefinitionData>),
	ExternType(Named<'a, ExternTypeDefinitionData>),
}

impl <'a> NamedTypeDefinition<'a> {
	pub fn name(&self) -> &'a QualifiedName {
		match self {
			NamedTypeDefinition::StructType(t) => t.name,
			NamedTypeDefinition::EnumType(t) => t.name,
			NamedTypeDefinition::ExternType(t) => t.name,
		}
	}

	pub fn type_params(&self) -> &'a Vec<String> {
		match self {
			NamedTypeDefinition::StructType(t) => &t.value.type_params,
			NamedTypeDefinition::EnumType(t) => &t.value.type_params,
			NamedTypeDefinition::ExternType(t) => &t.value.type_params,
		}
	}

	pub fn arity(&self) -> usize {
		self.type_params().len()
	}

	pub fn has_version(self, version: &BigUint) -> bool {
		match self {
			NamedTypeDefinition::StructType(t) => t.versioned(version).is_some(),
			NamedTypeDefinition::EnumType(t) => t.versioned(version).is_some(),
			NamedTypeDefinition::ExternType(_) => true,
		}
	}
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
	names: HashSet<QualifiedName>,
}

#[derive(Clone, Debug)]
pub enum ScopeLookup {
	NamedType(QualifiedName),
	TypeParameter(String),
}

pub struct Scope<'a> {
	model: &'a Verilization,
	current_pkg: Option<&'a PackageName>,
	imports: Option<&'a HashMap<String, ScopeLookup>>,
	type_params: Option<&'a Vec<String>>,
}

impl <'a> Scope<'a> {
	pub fn empty(model: &'a Verilization) -> Self {
		Scope {
			model: model,
			current_pkg: None,
			imports: None,
			type_params: None,
		}
	}

	pub fn lookup(&self, mut name: QualifiedName) -> ScopeLookup {
		if name.package.package.is_empty() {
			if let Some(type_params) = self.type_params {
				if type_params.contains(&name.name) {
					return ScopeLookup::TypeParameter(name.name);
				}
			}

			if let Some(import) = self.imports.and_then(|imports| imports.get(&name.name)) {
				return import.clone();
			}

			if let Some(current_pkg) = self.current_pkg {
				let current_pkg_name = QualifiedName {
					package: current_pkg.clone(),
					name: name.name,
				};
	
				if self.model.has_type(&current_pkg_name) {
					return ScopeLookup::NamedType(current_pkg_name);
				}
	
				name.name = current_pkg_name.name; // restore name because current_pkg_name is not a type
			}
		}

		ScopeLookup::NamedType(name)
	}

	pub fn resolve(&self, t: Type, type_args: &HashMap<String, Type>) -> Option<Type> {
		Some(match t {
			Type::Defined(name, args) => {
				match self.lookup(name) {
					ScopeLookup::NamedType(name) => {
						Type::Defined(name, args.into_iter().map(|arg| self.resolve(arg, type_args)).collect::<Option<Vec<_>>>()?)
					},
					ScopeLookup::TypeParameter(name) => {
						type_args.get(&name)?.clone()
					},
				}
			},
		})
	}
}

fn make_name_uppercase(name: &mut QualifiedName) {
	for part in &mut name.package.package {
		part.make_ascii_uppercase();
	}

	name.name.make_ascii_uppercase();
}

impl Verilization {

	/// Creates a new versioned format.
	pub fn new(metadata: VerilizationMetadata) -> Self {
		Verilization {
			metadata: metadata,
			constants: HashMap::new(),
			type_definitions: HashMap::new(),
			names: HashSet::new(),
		}
	}

	/// Adds a constant to the serialization format.
	pub fn add_constant(&mut self, name: QualifiedName, constant: Constant) -> Result<(), QualifiedName> {
		let mut case_name = name.clone();
		make_name_uppercase(&mut case_name);
		if self.names.insert(case_name) {
			self.constants.insert(name, constant);
			Ok(())
		}
		else {
			Err(name)
		}
	}

	/// Adds a type definition to the serialization format.
	pub fn add_type(&mut self, name: QualifiedName, t: TypeDefinition) -> Result<(), QualifiedName> {
		let mut case_name = name.clone();
		make_name_uppercase(&mut case_name);
		if self.names.insert(case_name) {
			self.type_definitions.insert(name, t);
			Ok(())
		}
		else {
			Err(name)
		}
	}

	pub fn get_type<'a>(&'a self, name: &QualifiedName) -> Option<NamedTypeDefinition<'a>> {
		let (name, t) = self.type_definitions.get_key_value(name)?;

		Some(match t {
			TypeDefinition::StructType(t) => NamedTypeDefinition::StructType(Named::new(self, name, t)),
			TypeDefinition::EnumType(t) => NamedTypeDefinition::EnumType(Named::new(self, name, t)),
			TypeDefinition::ExternType(t) => NamedTypeDefinition::ExternType(Named::new(self, name,  t)),
		})
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


	// Iterate constants.
	pub fn constants<'a>(&'a self) -> ConstantIterator<'a> {
		ConstantIterator {
			model: self,
			iter: self.constants.iter(),
		}
	}

	pub fn types<'a>(&'a self) -> TypeIterator<'a> {
		TypeIterator {
			model: self,
			iter: self.type_definitions.iter(),
		}
	}

}



// Iterators

pub struct ConstantIterator<'a> {
	model: &'a Verilization,
	iter: std::collections::hash_map::Iter<'a, QualifiedName, Constant>,
}

impl <'a> Iterator for ConstantIterator<'a> {
	type Item = Named<'a, Constant>;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|(name, constant)| Named::new(self.model, name, constant))
	}
}

pub struct ConstantVersionIterator<'a> {
	constant: Named<'a, Constant>,
	version: BigUint,
	max_version: BigUint,
	has_prev_version: bool,
}

impl <'a> Iterator for ConstantVersionIterator<'a> {
	type Item = ConstantVersionInfo<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		while self.version <= self.max_version {
			let version = self.version.clone();
			self.version += BigUint::one();
			
			if let Some(ver_type) = self.constant.value.versions.get(&version) {
				self.has_prev_version = true;
				return Some(ConstantVersionInfo {
					version: version,
					value: Some(ver_type),
					dummy: PhantomData {},
				});
			}
			else if self.has_prev_version {
				return Some(ConstantVersionInfo {
					version: version,
					value: None,
					dummy: PhantomData {},
				});
			}
		}

		None
	}
}

pub struct TypeIterator<'a> {
	model: &'a Verilization,
	iter: std::collections::hash_map::Iter<'a, QualifiedName, TypeDefinition>,
}

impl <'a> Iterator for TypeIterator<'a> {
	type Item = NamedTypeDefinition<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		self.iter.next().map(|(name, t)| match t {
			TypeDefinition::StructType(t) => NamedTypeDefinition::StructType(Named::new(self.model, name, t)),
			TypeDefinition::EnumType(t) => NamedTypeDefinition::EnumType(Named::new(self.model, name, t)),
			TypeDefinition::ExternType(t) => NamedTypeDefinition::ExternType(Named::new(self.model, name, t)),
		})
	}
}

pub struct TypeVersionIterator<'a> {
	type_def: Named<'a, VersionedTypeDefinitionData>,
	version: BigUint,
	max_version: BigUint,
	last_seen_version: Option<&'a TypeVersionDefinition>,
}

impl <'a> Iterator for TypeVersionIterator<'a> {
	type Item = TypeVersionInfo<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		while self.version <= self.max_version {
			let version = self.version.clone();
			self.version += BigUint::one();
			
			if let Some(ver_type) = self.type_def.value.versions.get(&version) {
				self.last_seen_version = Some(ver_type);
				return Some(TypeVersionInfo {
					version: version,
					explicit_version: true,
					ver_type: ver_type,
					dummy: PhantomData {},
				});
			}
			else if let Some(ver_type) = self.last_seen_version {
				return Some(TypeVersionInfo {
					version: version,
					explicit_version: false,
					ver_type: ver_type,
					dummy: PhantomData {},
				});
			}
		}

		None
	}
}


pub struct ReferencedTypeIterator<'a> {
	seen_types: HashSet<&'a QualifiedName>,
	ver_iter: std::collections::hash_map::Values<'a, BigUint, TypeVersionDefinition>,
	field_iter: std::slice::Iter<'a, (String, FieldInfo)>,
	arg_iters: Vec<std::slice::Iter<'a, Type>>,
}

lazy_static! {
	static ref REF_TYPE_ITER_EMPTY_VER_MAP: HashMap<BigUint, TypeVersionDefinition> = HashMap::new();
}
const REF_TYPE_ITER_EMPTY_FIELD_SLICE: &[(String, FieldInfo)] = &[];

impl <'a> Iterator for ReferencedTypeIterator<'a> {
	type Item = &'a QualifiedName;

	fn next(&mut self) -> Option<Self::Item> {
		loop {
			while let Some(arg_iter) = self.arg_iters.last_mut() {
				if let Some(arg) = arg_iter.next() {
					let Type::Defined(name, args) = arg;
					self.arg_iters.push(args.iter());
					if self.seen_types.insert(name) {
						return Some(name);
					}
			}
				else {
					self.arg_iters.pop();
				}
			}

			if let Some((_, field)) = self.field_iter.next() {
				self.arg_iters.push(std::slice::from_ref(&field.field_type).iter());
			}
			else if let Some(ver_type) = self.ver_iter.next() {
				self.field_iter = ver_type.fields.iter();
			}
			else {
				return None;
			}
		}
	}
}

impl <'a> ReferencedTypeIterator<'a> {
	fn from_versions(versions: &'a HashMap<BigUint, TypeVersionDefinition>) -> ReferencedTypeIterator<'a> {
		ReferencedTypeIterator {
			seen_types: HashSet::new(),
			ver_iter: versions.values(),
			field_iter: REF_TYPE_ITER_EMPTY_FIELD_SLICE.iter(),
			arg_iters: Vec::new(),
		}
	}

	fn from_type(t: &'a Type) -> ReferencedTypeIterator<'a> {
		ReferencedTypeIterator {
			seen_types: HashSet::new(),
			ver_iter: REF_TYPE_ITER_EMPTY_VER_MAP.values(),
			field_iter: REF_TYPE_ITER_EMPTY_FIELD_SLICE.iter(),
			arg_iters: vec!(std::slice::from_ref(t).iter()),
		}
	}
}