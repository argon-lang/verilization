use num_bigint::{ BigUint, BigInt };
use num_traits::{Zero, One};
use std::collections::{HashMap, HashSet};
use std::fmt;
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

#[derive(Debug)]
pub enum ModelError {
	DuplicateConstant(QualifiedName),
	DuplicateType(QualifiedName),
	DuplicateVersion(QualifiedName, BigUint),
	DuplicateField(QualifiedName, BigUint, String),
	DuplicateLiteralInteger(QualifiedName),
	DuplicateLiteralString(QualifiedName),
	DuplicateLiteralSequence(QualifiedName),
	DuplicateLiteralCase(QualifiedName, String),
	DuplicateLiteralRecord(QualifiedName),
	DuplicateLiteralRecordField(QualifiedName, String),
	DuplicateFieldValue(String),
}

impl fmt::Display for ModelError {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		match self {
			ModelError::DuplicateConstant(name) => write!(f, "Cannot declare constant {}. Name is already defined.", name),
			ModelError::DuplicateType(name) => write!(f, "Cannot declare type {}. Name is already defined.", name),
			ModelError::DuplicateVersion(name, version) => write!(f, "Version {} of type {} is already defined.", version, name),
			ModelError::DuplicateField(name, version, field) => write!(f, "Version {} of type {} already has a field named {}.", version, name, field),
			ModelError::DuplicateLiteralInteger(type_name) => write!(f, "Type {} already has an integer literal.", type_name),
			ModelError::DuplicateLiteralString(type_name) => write!(f, "Type {} already has a string literal.", type_name),
			ModelError::DuplicateLiteralSequence(type_name) => write!(f, "Type {} already has a sequence literal.", type_name),
			ModelError::DuplicateLiteralCase(type_name, name) => write!(f, "Type {} already has a literal for case {}.", type_name, name),
			ModelError::DuplicateLiteralRecord(type_name) => write!(f, "Type {} already has a record literal.", type_name),
			ModelError::DuplicateLiteralRecordField(type_name, field) => write!(f, "Record literal for type {} already has a field named {}.", type_name, field),
			ModelError::DuplicateFieldValue(name) => write!(f, "Record constant already has a field named {}.", name),
		}
	}
}

/// A data type. This includes the name of the type and the type arguments.
#[derive(Clone, Debug)]
pub struct Type {
	pub name: QualifiedName,
	pub args: Vec<Type>,
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

	/// Gets the name of this named value.
	pub fn name(&self) -> &'a QualifiedName {
		self.name
	}
}

/// The value of a constant.
#[derive(Clone, Debug)]
pub enum ConstantValue {
	Integer(BigInt),
	String(String),
	Sequence(Vec<ConstantValue>),
	Case(String, Vec<ConstantValue>),
	Record(ConstantValueRecord),
	Constant(QualifiedName),
}

#[derive(Clone, Debug)]
pub struct ConstantValueRecord {
	field_values: HashMap<String, ConstantValue>,
}

impl ConstantValueRecord {
	pub fn field_values<'a>(&'a self) -> &'a HashMap<String, ConstantValue> {
		&self.field_values
	}

	pub fn into_field_values(self) -> HashMap<String, ConstantValue> {
		self.field_values
	}
}

pub struct ConstantValueRecordBuilder {
	field_names: HashSet<String>,
	record: ConstantValueRecord,
}

impl ConstantValueRecordBuilder {
	pub fn new() -> Self {
		ConstantValueRecordBuilder {
			field_names: HashSet::new(),
			record: ConstantValueRecord {
				field_values: HashMap::new(),
			},
		}
	}

	pub fn add_field(&mut self, name: String, value: ConstantValue) -> Result<(), ModelError> {
		if self.field_names.insert(name.to_ascii_uppercase()) {
			self.record.field_values.insert(name, value);
			Ok(())
		}
		else {
			Err(ModelError::DuplicateFieldValue(name))
		}
	}

	pub fn build(self) -> ConstantValueRecord {
		self.record
	}
}


/// The result of looking up a constant for a specific format version.
/// If value is None, then the value was defined in a previous version.
pub struct ConstantVersionInfo<'a> {
	pub version: BigUint,
	pub explicit_version: bool,
	pub value: &'a ConstantValue,

	dummy: PhantomData<()>,
}

/// A constant definition.
/// 
/// See accessor methods for [`Named`] constants.
pub struct Constant {
	latest_version: BigUint,
	imports: HashMap<String, QualifiedName>,
	value_type: Type,
	versions: HashMap<BigUint, ConstantValue>,
}

pub struct ConstantBuilder {
	name: QualifiedName,
	constant: Constant,
}

impl ConstantBuilder {
	pub fn new(latest_version: BigUint, name: QualifiedName, value_type: Type, imports: HashMap<String, QualifiedName>) -> Self {
		ConstantBuilder {
			name: name,
			constant: Constant {
				latest_version: latest_version,
				imports: imports,
				value_type: value_type,
				versions: HashMap::new(),
			},
		}
	}

	pub fn add_version(&mut self, version: BigUint, value: ConstantValue) -> Result<(), ModelError> {
		if self.constant.versions.contains_key(&version) {
			Err(ModelError::DuplicateVersion(self.name.clone(), version.clone()))
		}
		else {
			self.constant.versions.insert(version, value);
			Ok(())
		}
	}
}

impl <'a> Named<'a, Constant> {

	/// The type of the constant.
	pub fn value_type(self) -> &'a Type {
		&self.value.value_type
	}

	/// Iterates over types referenced in the type of the constant.
	pub fn referenced_types(self) -> ReferencedTypeIterator<'a> {
		ReferencedTypeIterator::from_type(&self.value.value_type)
	}

	/// Iterates over the versions of the constant from the first version to the latest version of the model.
	pub fn versions(self) -> ConstantVersionIterator<'a> {
		ConstantVersionIterator {
			constant: self,
			version: BigUint::one(),
			last_seen_version: None,
			max_version: self.value.latest_version.clone(),
		}
	}

	/// Gets a scope for this constant element.
	pub fn scope(self) -> Scope<'a> {
		Scope {
			model: self.model,
			current_pkg: Some(&self.name.package),
			imports: Some(&self.value.imports),
			type_params: None,
		}
	}

	/// Gets a version of this constant.
	pub fn versioned(self, version: &BigUint) -> Option<ConstantVersionInfo<'a>> {
		if version > &self.value.latest_version {
			None
		}
		else {
			self.value.versions.iter()
				.filter(|(ver, _)| ver <= &version)
				.max_by_key(|(ver, _)| ver.clone())
				.map(|(actual_ver, value)| {
					ConstantVersionInfo {
						version: version.clone(),
						explicit_version: version == actual_ver,
						value: value,
						dummy: PhantomData {},
					}
				})
		}
	}

	/// Returns true if the constant exists in the specified version.
	pub fn has_version(self, version: &BigUint) -> bool {
		self.versioned(version).is_some()
	}

}

/// A field of a struct or enum. An enum field represents a single case.
#[derive(Debug)]
pub struct FieldInfo {
	pub field_type: Type,
}

/// A versioned type defines the contents of a type for a specific format version.
#[derive(Debug)]
pub struct TypeVersionDefinition {
	fields: Vec<(String, FieldInfo)>,
}

impl TypeVersionDefinition {
	pub fn fields(&self) -> &Vec<(String, FieldInfo)> {
		&self.fields
	}
}

pub struct TypeVersionDefinitionBuilder<'a> {
	name: &'a QualifiedName,
	version: BigUint,
	field_names: HashSet<String>,
	ver_type: &'a mut TypeVersionDefinition,
}

impl <'a> TypeVersionDefinitionBuilder<'a> {
	pub fn add_field(&mut self, name: String, field: FieldInfo) -> Result<(), ModelError> {
		if self.field_names.insert(name.to_ascii_uppercase()) {
			self.ver_type.fields.push((name, field));
			Ok(())
		}
		else {
			Err(ModelError::DuplicateField(self.name.clone(), self.version.clone(), name))
		}
	}
}

/// The result of looking up a version of a type.
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

/// Defines a versioned type. Could be a struct or enum.
#[derive(Debug)]
pub struct VersionedTypeDefinitionData {
	latest_version: BigUint,
	imports: HashMap<String, QualifiedName>,
	type_params: Vec<String>,
	versions: HashMap<BigUint, TypeVersionDefinition>,
	is_final: bool,
}

pub struct VersionedTypeDefinitionBuilder {
	name: QualifiedName,
	t: VersionedTypeDefinitionData,
}

impl VersionedTypeDefinitionBuilder {
	pub fn new(latest_version: BigUint, name: QualifiedName, type_params: Vec<String>, is_final: bool, imports: HashMap<String, QualifiedName>) -> Self {
		VersionedTypeDefinitionBuilder {
			name: name,
			t: VersionedTypeDefinitionData {
				latest_version: latest_version,
				imports: imports,
				type_params: type_params,
				is_final: is_final,
				versions: HashMap::new(),
			},
		}
	}

	pub fn add_version<'a>(&'a mut self, version: BigUint) -> Result<TypeVersionDefinitionBuilder<'a>, ModelError> {
		match self.t.versions.entry(version.clone()) {
			std::collections::hash_map::Entry::Occupied(_) => Err(ModelError::DuplicateVersion(self.name.clone(), version)),
			std::collections::hash_map::Entry::Vacant(entry) => {
				let ver_type = entry.insert(TypeVersionDefinition {
					fields: Vec::new(),
				});

				Ok(TypeVersionDefinitionBuilder {
					name: &self.name,
					version: version,
					field_names: HashSet::new(),
					ver_type: ver_type,
				})
			}
		}
	}
}

impl <'a> Named<'a, VersionedTypeDefinitionData> {

	/// Gets whether this type is final.
	pub fn is_final(&self) -> bool {
		self.value.is_final
	}

	/// Gets a version of this type.
	pub fn versioned(self, version: &BigUint) -> Option<TypeVersionInfo<'a>> {
		if version > &self.value.latest_version && !self.value.is_final {
			None
		}
		else {
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
	}

	/// Finds the last explicitly defined version of this type.
	pub fn last_explicit_version(self) -> Option<&'a BigUint> {
		self.value.versions.keys().max()
	}

	/// Iterates over types referenced in the field types of this type.
	pub fn referenced_types(self) -> ReferencedTypeIterator<'a> {
		ReferencedTypeIterator::from_versions(&self.value.versions)
	}

	/// Iterates over the versions of this type.
	/// 
	/// Starts at the first version of the type.
	/// If the type is final, ends at the last explicitly defined version.
	/// If the type is not final, ends at the latest version of the model.
	pub fn versions(self) -> TypeVersionIterator<'a> {
		TypeVersionIterator {
			type_def: self,
			version: BigUint::one(),
			max_version:
				if self.value.is_final {
					self.last_explicit_version().map(|ver| ver.clone()).unwrap_or(BigUint::zero())
				}
				else {
					self.value.latest_version.clone()
				},

			last_seen_version: None,
		}
	}

	/// Gets a scope for the type.
	pub fn scope(self) -> Scope<'a> {
		Scope {
			model: self.model,
			current_pkg: Some(&self.name.package),
			imports: Some(&self.value.imports),
			type_params: Some(&self.value.type_params),
		}
	}

	/// Gets the parameters of the type.
	pub fn type_params(self) -> &'a Vec<String> {
		&self.value.type_params
	}
}

/// Defines an extern type.
#[derive(Debug)]
pub struct ExternTypeDefinitionData {
	imports: HashMap<String, QualifiedName>,
	type_params: Vec<String>,
	literals: Vec<ExternLiteralSpecifier>,
}

pub struct ExternTypeDefinitionBuilder {
	name: QualifiedName,
	has_integer: bool,
	has_string: bool,
	has_sequence: bool,
	cases: HashSet<String>,
	has_record: bool,
	t: ExternTypeDefinitionData,
}

pub struct ExternLiteralRecordBuilder<'a> {
	name: &'a QualifiedName,
	field_names: HashSet<String>,
	fields: &'a mut Vec<(String, FieldInfo)>,
}

impl ExternTypeDefinitionBuilder {
	pub fn new(name: QualifiedName, type_params: Vec<String>, imports: HashMap<String, QualifiedName>) -> Self {
		ExternTypeDefinitionBuilder {
			name: name,
			has_integer: false,
			has_string: false,
			has_sequence: false,
			cases: HashSet::new(),
			has_record: false,
			t: ExternTypeDefinitionData {
				imports: imports,
				type_params: type_params,
				literals: Vec::new(),
			},
		}
	}
	
	pub fn add_integer_literal(&mut self, lower_type: ExternLiteralIntBound, lower: Option<BigInt>, upper_type: ExternLiteralIntBound, upper: Option<BigInt>) -> Result<(), ModelError> {
		if self.has_integer {
			Err(ModelError::DuplicateLiteralInteger(self.name.clone()))
		}
		else {
			self.t.literals.push(ExternLiteralSpecifier::Integer(lower_type, lower, upper_type, upper));
			self.has_integer = true;
			Ok(())
		}
	}
	
	pub fn add_string_literal(&mut self) -> Result<(), ModelError> {
		if self.has_string {
			Err(ModelError::DuplicateLiteralString(self.name.clone()))
		}
		else {
			self.t.literals.push(ExternLiteralSpecifier::String);
			self.has_string = true;
			Ok(())
		}
	}
	
	pub fn add_sequence_literal(&mut self, element_type: Type) -> Result<(), ModelError> {
		if self.has_sequence {
			Err(ModelError::DuplicateLiteralSequence(self.name.clone()))
		}
		else {
			self.t.literals.push(ExternLiteralSpecifier::Sequence(element_type));
			self.has_sequence = true;
			Ok(())
		}
	}
	
	pub fn add_case_literal(&mut self, case_name: String, params: Vec<Type>) -> Result<(), ModelError> {
		if self.cases.insert(case_name.to_ascii_uppercase()) {
			self.t.literals.push(ExternLiteralSpecifier::Case(case_name, params));
			Ok(())
		}
		else {
			Err(ModelError::DuplicateLiteralCase(self.name.clone(), case_name))
		}
	}
	
	pub fn add_record_literal<'a>(&'a mut self) -> Result<ExternLiteralRecordBuilder<'a>, ModelError> {
		if self.has_record {
			Err(ModelError::DuplicateLiteralRecord(self.name.clone()))
		}
		else {
			self.has_record = true;
			self.t.literals.push(ExternLiteralSpecifier::Record(Vec::new()));
			let fields = match self.t.literals.last_mut() {
				Some(ExternLiteralSpecifier::Record(fields)) => fields,
				_ => panic!("Last element should have been a record"),
			};
			Ok(ExternLiteralRecordBuilder {
				name: &self.name,
				field_names: HashSet::new(),
				fields: fields,
			})
		}
	}


}

impl <'a> ExternLiteralRecordBuilder<'a> {
	pub fn add_field(&mut self, name: String, field: FieldInfo) -> Result<(), ModelError> {
		if self.field_names.insert(name.to_ascii_uppercase()) {
			self.fields.push((name, field));
			Ok(())
		}
		else {
			Err(ModelError::DuplicateLiteralRecordField(self.name.clone(), name))
		}
	}
}

/// Defines a bound for an integer literal definition.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ExternLiteralIntBound {
	Inclusive,
	Exclusive,
}

/// Defines a literal for an extern type.
#[derive(Debug)]
pub enum ExternLiteralSpecifier {
	Integer(ExternLiteralIntBound, Option<BigInt>, ExternLiteralIntBound, Option<BigInt>),
	String,
	Sequence(Type),
	Case(String, Vec<Type>),
	Record(Vec<(String, FieldInfo)>),
}

impl <'a> Named<'a, ExternTypeDefinitionData> {
	/// Get the literals defined by the type.
	pub fn literals(self) -> &'a Vec<ExternLiteralSpecifier> {
		&self.value.literals
	}

	/// Gets a scope for the type.
	pub fn scope(self) -> Scope<'a> {
		Scope {
			model: self.model,
			current_pkg: Some(&self.name.package),
			imports: Some(&self.value.imports),
			type_params: Some(&self.value.type_params),
		}
	}

	/// Gets the parameters for the type.
	pub fn type_params(self) -> &'a Vec<String> {
		&self.value.type_params
	}
}



/// A definition of a type.
pub enum TypeDefinition {
	StructType(VersionedTypeDefinitionData),
	EnumType(VersionedTypeDefinitionData),
	ExternType(ExternTypeDefinitionData),
}

/// A named definition of a type.
#[derive(Copy, Clone)]
pub enum NamedTypeDefinition<'a> {
	StructType(Named<'a, VersionedTypeDefinitionData>),
	EnumType(Named<'a, VersionedTypeDefinitionData>),
	ExternType(Named<'a, ExternTypeDefinitionData>),
}

impl <'a> NamedTypeDefinition<'a> {
	/// Get the name of the type.
	pub fn name(&self) -> &'a QualifiedName {
		match self {
			NamedTypeDefinition::StructType(t) => t.name,
			NamedTypeDefinition::EnumType(t) => t.name,
			NamedTypeDefinition::ExternType(t) => t.name,
		}
	}

	/// Gets the parameters of the type.
	pub fn type_params(&self) -> &'a Vec<String> {
		match self {
			NamedTypeDefinition::StructType(t) => &t.value.type_params,
			NamedTypeDefinition::EnumType(t) => &t.value.type_params,
			NamedTypeDefinition::ExternType(t) => &t.value.type_params,
		}
	}

	/// Gets the number of parameters of the type.
	pub fn arity(&self) -> usize {
		self.type_params().len()
	}

	/// Returns true if the type exists in the specified version.
	pub fn has_version(self, version: &BigUint) -> bool {
		match self {
			NamedTypeDefinition::StructType(t) => t.versioned(version).is_some(),
			NamedTypeDefinition::EnumType(t) => t.versioned(version).is_some(),
			NamedTypeDefinition::ExternType(_) => true,
		}
	}

	/// Gets a scope for this type.
	pub fn scope(self) -> Scope<'a> {
		match self {
			NamedTypeDefinition::StructType(t) => t.scope(),
			NamedTypeDefinition::EnumType(t) => t.scope(),
			NamedTypeDefinition::ExternType(t) => t.scope(),
		}
	}
}

/// Defines a versioned serialization format.
pub struct Verilization {
	constants: HashMap<QualifiedName, Constant>,
	type_definitions: HashMap<QualifiedName, TypeDefinition>,
	names: HashSet<QualifiedName>,
}

/// The result of looking up a name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeLookup {
	NamedType(QualifiedName),
	TypeParameter(String),
}

/// A Scope allows looking up names.
/// 
/// It can identify names that are type parameters, names in the current package, etc.
pub struct Scope<'a> {
	model: &'a Verilization,
	current_pkg: Option<&'a PackageName>,
	imports: Option<&'a HashMap<String, QualifiedName>>,
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
				return ScopeLookup::NamedType(import.clone());
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

	pub fn lookup_constant(&self, mut name: QualifiedName) -> QualifiedName {
		if name.package.package.is_empty() {
			if let Some(import) = self.imports.and_then(|imports| imports.get(&name.name)) {
				return import.clone();
			}

			if let Some(current_pkg) = self.current_pkg {
				let current_pkg_name = QualifiedName {
					package: current_pkg.clone(),
					name: name.name,
				};
	
				if self.model.has_constant(&current_pkg_name) {
					return current_pkg_name;
				}
	
				name.name = current_pkg_name.name; // restore name because current_pkg_name is not a type
			}
		}

		name
	}

	pub fn type_params(&self) -> Vec<String> {
		match self.type_params {
			Some(params) => params.clone(),
			None => Vec::new(),
		}
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
	pub fn new() -> Self {
		Verilization {
			constants: HashMap::new(),
			type_definitions: HashMap::new(),
			names: HashSet::new(),
		}
	}

	/// Adds a constant to the serialization format.
	pub fn add_constant(&mut self, constant: ConstantBuilder) -> Result<(), ModelError> {
		let mut case_name = constant.name.clone();
		make_name_uppercase(&mut case_name);
		if self.names.insert(case_name) {
			self.constants.insert(constant.name, constant.constant);
			Ok(())
		}
		else {
			Err(ModelError::DuplicateConstant(constant.name))
		}
	}

	/// Adds a struct to the serialization format.
	pub fn add_struct_type(&mut self, type_def: VersionedTypeDefinitionBuilder) -> Result<(), ModelError> {
		let mut case_name = type_def.name.clone();
		make_name_uppercase(&mut case_name);
		if self.names.insert(case_name) {
			self.type_definitions.insert(type_def.name, TypeDefinition::StructType(type_def.t));
			Ok(())
		}
		else {
			Err(ModelError::DuplicateType(type_def.name))
		}
	}

	/// Adds an enum to the serialization format.
	pub fn add_enum_type(&mut self, type_def: VersionedTypeDefinitionBuilder) -> Result<(), ModelError> {
		let mut case_name = type_def.name.clone();
		make_name_uppercase(&mut case_name);
		if self.names.insert(case_name) {
			self.type_definitions.insert(type_def.name, TypeDefinition::EnumType(type_def.t));
			Ok(())
		}
		else {
			Err(ModelError::DuplicateType(type_def.name))
		}
	}

	/// Adds an extern to the serialization format.
	pub fn add_extern_type(&mut self, type_def: ExternTypeDefinitionBuilder) -> Result<(), ModelError> {
		let mut case_name = type_def.name.clone();
		make_name_uppercase(&mut case_name);
		if self.names.insert(case_name) {
			self.type_definitions.insert(type_def.name, TypeDefinition::ExternType(type_def.t));
			Ok(())
		}
		else {
			Err(ModelError::DuplicateType(type_def.name))
		}
	}

	/// Finds a constant in the model.
	pub fn get_constant<'a>(&'a self, name: &QualifiedName) -> Option<Named<'a, Constant>> {
		let (name, constant) = self.constants.get_key_value(name)?;

		Some(Named::new(self, name, constant))
	}

	/// Finds a type in the model.
	pub fn get_type<'a>(&'a self, name: &QualifiedName) -> Option<NamedTypeDefinition<'a>> {
		let (name, t) = self.type_definitions.get_key_value(name)?;

		Some(match t {
			TypeDefinition::StructType(t) => NamedTypeDefinition::StructType(Named::new(self, name, t)),
			TypeDefinition::EnumType(t) => NamedTypeDefinition::EnumType(Named::new(self, name, t)),
			TypeDefinition::ExternType(t) => NamedTypeDefinition::ExternType(Named::new(self, name,  t)),
		})
	}

	/// Determines whether a type with this name exists.
	pub fn has_type(&self, name: &QualifiedName) -> bool {
		self.type_definitions.contains_key(name)
	}

	pub fn has_constant(&self, name: &QualifiedName) -> bool {
		self.constants.contains_key(name)
	}

	/// Merges two serialization formats.
	pub fn merge(&mut self, other: Verilization) -> Result<(), ModelError> {
		other.constants.into_iter().try_for_each(|(name, constant)| self.add_constant(ConstantBuilder { name: name, constant: constant }))?;
		other.type_definitions.into_iter().try_for_each(|(name, t)| match t {
			TypeDefinition::StructType(type_def) => self.add_struct_type(VersionedTypeDefinitionBuilder { name: name, t: type_def }),
			TypeDefinition::EnumType(type_def) => self.add_enum_type(VersionedTypeDefinitionBuilder { name: name, t: type_def }),
			TypeDefinition::ExternType(type_def) => self.add_extern_type(ExternTypeDefinitionBuilder { name: name, t: type_def, has_integer: false, has_string: false, has_sequence: false, cases: HashSet::new(), has_record: false, }),
		})?;

		Ok(())
	}


	/// Iterate over constants defined in the model.
	pub fn constants<'a>(&'a self) -> ConstantIterator<'a> {
		ConstantIterator {
			model: self,
			iter: self.constants.iter(),
		}
	}

	/// Iterate over types defined in the model.
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
	last_seen_version: Option<&'a ConstantValue>,
}

impl <'a> Iterator for ConstantVersionIterator<'a> {
	type Item = ConstantVersionInfo<'a>;

	fn next(&mut self) -> Option<Self::Item> {
		while self.version <= self.max_version {
			let version = self.version.clone();
			self.version += BigUint::one();
			
			if let Some(ver_const) = self.constant.value.versions.get(&version) {
				self.last_seen_version = Some(ver_const);
				return Some(ConstantVersionInfo {
					version: version,
					value: ver_const,
					dummy: PhantomData {},
					explicit_version: true,
				});
			}
			else if let Some(ver_const) = self.last_seen_version {
				return Some(ConstantVersionInfo {
					version: version,
					value: ver_const,
					dummy: PhantomData {},
					explicit_version: false,
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
					self.arg_iters.push(arg.args.iter());
					if self.seen_types.insert(&arg.name) {
						return Some(&arg.name);
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