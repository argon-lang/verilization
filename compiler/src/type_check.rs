use crate::model::*;
use num_bigint::BigUint;
use num_traits::One;
use std::collections::{HashSet, HashMap};

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub enum TypeCheckError {
    TypeNotDefined(QualifiedName),
    ConstantNotDefined(QualifiedName),
    TypeNotInVersion(QualifiedName, BigUint),
    ConstantNotInVersion(QualifiedName, BigUint),
    ConstantHasIncorrectType(QualifiedName, BigUint),
    CouldNotFindLastVersion(QualifiedName),
    ArityMismatch(usize, usize),
    TypeNotFinal(QualifiedName),
    DuplicateLiteral(QualifiedName),
}

struct TypeCheck<'model> {
    model: &'model Verilization,
    scope: Scope<'model>,
}

fn try_all<I: Iterator, E>(iter: I, mut f: impl FnMut(I::Item) -> Result<bool, E>) -> Result<bool, E> {
    for value in iter {
        if !f(value)? {
            return Ok(false)
        }
    }
    Ok(true)
}

fn try_any<I: Iterator, E>(iter: I, mut f: impl FnMut(I::Item) -> Result<bool, E>) -> Result<bool, E> {
    for value in iter {
        if f(value)? {
            return Ok(true)
        }
    }
    Ok(false)
}

fn check_record(tc: &TypeCheck, version: &BigUint, record: &ConstantValueRecord, record_def: &Vec<(String, FieldInfo)>) -> Result<bool, TypeCheckError> {
    let mut value_map = HashMap::new();
    for (field_name, value) in record.field_values() {
        value_map.insert(field_name, value);
    }

    for (field_name, field) in record_def {
        if let Some(field_value) = value_map.remove(field_name) {
            if !tc.check_value_type(version, &field.field_type, field_value)? {
                return Ok(false)
            }
        }
        else {
            return Ok(false)
        }
    }
    
    Ok(value_map.is_empty())
}


fn same_types(a: &Type, a_scope: &Scope, b: &Type, b_scope: &Scope) -> bool {
    if a.args.len() != b.args.len() {
        return false
    }

    let a_res = a_scope.lookup(a.name.clone());
    let b_res = b_scope.lookup(b.name.clone());
    
    if a_res != b_res {
        return false
    }

    a.args.iter().zip(b.args.iter())
        .all(|(a_arg, b_arg)| same_types(a_arg, a_scope, b_arg, b_scope))
}

impl <'model> TypeCheck<'model> {

    fn check_type(&self, version: &BigUint, t: &Type) -> Result<(), TypeCheckError> {
        match self.scope.lookup(t.name.clone()) {
            ScopeLookup::NamedType(name) => {
                let named_type_def = match self.model.get_type(&name) {
                    Some(t) => t,
                    None => return Err(TypeCheckError::TypeNotDefined(name)),
                };

                if !named_type_def.has_version(version) {
                    return Err(TypeCheckError::TypeNotInVersion(name, version.clone()));
                }
    
                let arity = named_type_def.arity();
                if arity != t.args.len() {
                    return Err(TypeCheckError::ArityMismatch(arity, t.args.len()));
                }
    
                for arg in &t.args {
                    self.check_type(version, &arg)?;
                }

                Ok(())
            },
            ScopeLookup::TypeParameter(_) => {
                if t.args.len() != 0 {
                    return Err(TypeCheckError::ArityMismatch(0, t.args.len()));
                }

                Ok(())
            }
        }
    }

    fn check_is_final(&self, version: &BigUint, t: &Type) -> Result<bool, TypeCheckError> {
        Ok(match self.scope.lookup(t.name.clone()) {
            ScopeLookup::NamedType(name) => {
                match self.model.get_type(&name).ok_or_else(|| TypeCheckError::TypeNotDefined(name.clone()))? {
                    NamedTypeDefinition::StructType(type_def) | NamedTypeDefinition::EnumType(type_def) => {
                        if !type_def.is_final() {
                            return Ok(false);
                        }
                        
                        if !(type_def.last_explicit_version().ok_or_else(|| TypeCheckError::CouldNotFindLastVersion(name.clone()))? <= version) {
                            return Ok(false);
                        }
                    },

                    NamedTypeDefinition::ExternType(_) => (),
                }

                for arg in &t.args {
                    if !self.check_is_final(version, &arg)? {
                        return Ok(false);
                    }
                }

                true
            },
            ScopeLookup::TypeParameter(_) => true,
        })
    }

    fn check_value_type(&self, version: &BigUint, t: &Type, value: &ConstantValue) -> Result<bool, TypeCheckError> {
        let (type_name, named_type_def) = match self.scope.lookup(t.name.clone()) {
            ScopeLookup::NamedType(name) => match self.model.get_type(&name) {
                Some(t) => (name, t),
                None => return Err(TypeCheckError::TypeNotDefined(name)),
            },
            ScopeLookup::TypeParameter(_) => return Ok(false)
        };


        match (value, named_type_def) {
            (ConstantValue::Integer(n), NamedTypeDefinition::ExternType(extern_type)) =>
                Ok(extern_type.literals().iter().any(|literal| match literal {
                    ExternLiteralSpecifier::Integer(lower_bound, lower_value, upper_bound, upper_value) =>
                        (
                            match (lower_bound, lower_value) {
                                (_, None) => true,
                                (ExternLiteralIntBound::Inclusive, Some(x)) => n >= x,
                                (ExternLiteralIntBound::Exclusive, Some(x)) => n > x,
                            }
                        ) && (
                            match (upper_bound, upper_value) {
                                (_, None) => true,
                                (ExternLiteralIntBound::Inclusive, Some(x)) => n <= x,
                                (ExternLiteralIntBound::Exclusive, Some(x)) => n < x,
                            }
                        ),

                    _ => false,
                })),

            (ConstantValue::Integer(_), _) => Ok(false),

            (ConstantValue::String(_), NamedTypeDefinition::ExternType(extern_type)) =>
                Ok(extern_type.literals().iter().any(|literal| match literal {
                    ExternLiteralSpecifier::String => true,
                    _ => false,
                })),

            (ConstantValue::String(_), _) => Ok(false),

            (ConstantValue::Sequence(seq), NamedTypeDefinition::ExternType(extern_type)) =>
                try_any(extern_type.literals().iter(), |literal| match literal {
                    ExternLiteralSpecifier::Sequence(elem_type) => {
                        try_all(seq.iter(), |elem| self.check_value_type(version, elem_type, elem))
                    },
                    _ => Ok(false),
                }),

            (ConstantValue::Sequence(_), _) => Ok(false),

            (ConstantValue::Case(name, args), NamedTypeDefinition::ExternType(extern_type)) =>
                try_any(extern_type.literals().iter(), |literal| match literal {
                    ExternLiteralSpecifier::Case(name2, param_types) if name == name2 && args.len() == param_types.len() =>
                        try_all(args.iter().zip(param_types.iter()), |(arg, param_type)| self.check_value_type(version, param_type, arg)),
                    _ => Ok(false),
                }),

            (ConstantValue::Case(name, args), NamedTypeDefinition::EnumType(enum_type)) if args.len() == 1 => {
                if let Some(type_ver) = enum_type.versioned(version) {
                    if let Some(field) = type_ver.ver_type.fields().iter().find_map(|(field_name, field)| if field_name == name { Some(field) } else { None }) {
                        self.check_value_type(version, &field.field_type, &args[0])
                    }
                    else {
                        Ok(false)
                    }
                }
                else {
                    Err(TypeCheckError::TypeNotInVersion(type_name, version.clone()))
                }
            },

            (ConstantValue::Case(_, _), _) => Ok(false),

            (ConstantValue::Record(record), NamedTypeDefinition::ExternType(extern_type)) =>
                try_any(extern_type.literals().iter(), |literal| match literal {
                    ExternLiteralSpecifier::Record(record_def) => check_record(self, version, record, record_def),
                    _ => Ok(false),
                }),

            (ConstantValue::Record(record), NamedTypeDefinition::StructType(struct_type)) => {
                if let Some(type_ver) = struct_type.versioned(version) {
                    check_record(self, version, record, type_ver.ver_type.fields())
                }
                else {
                    Err(TypeCheckError::TypeNotInVersion(type_name, version.clone()))
                }
            },

            (ConstantValue::Record(_), _) => Ok(false),

            (ConstantValue::Constant(constant_name), _) => {
                let constant_name = self.scope.lookup_constant(constant_name.clone());
                let constant = match self.model.get_constant(&constant_name) {
                    Some(constant) => constant,
                    None => return Err(TypeCheckError::ConstantNotDefined(constant_name)),
                };

                if !constant.has_version(version) {
                    return Err(TypeCheckError::ConstantNotInVersion(constant_name, version.clone()))
                }

                Ok(same_types(t, &self.scope, constant.value_type(), &constant.scope()))
            }
                
        }
    }
    
}

fn type_check_versioned_type<'model>(model: &'model Verilization, t: Named<'model, VersionedTypeDefinitionData>) -> Result<(), TypeCheckError> {
    let tc = TypeCheck {
        model: model,
        scope: t.scope(),
    };

    for ver in t.versions() {
        for (_, field) in ver.ver_type.fields() {
            tc.check_type(&ver.version, &field.field_type)?;
        }
    }

    if t.is_final() {
        if let Some(last_ver) = t.versions().last() {
            for (_, field) in last_ver.ver_type.fields() {
                if !tc.check_is_final(&last_ver.version, &field.field_type)? {
                    return Err(TypeCheckError::TypeNotFinal(t.name().clone()))
                }
            }
        }
    }

    Ok(())
}

fn type_check_extern_type<'model>(model: &'model Verilization, t: Named<'model, ExternTypeDefinitionData>) -> Result<(), TypeCheckError> {
    let tc = TypeCheck {
        model: model,
        scope: t.scope(),
    };

    let mut has_integer = false;
    let mut has_string = false;
    let mut has_sequence = false;
    let mut literal_cases = HashSet::new();
    let mut has_record = false;

    for literal in t.literals() {
        match literal {
            ExternLiteralSpecifier::Integer(_, _, _, _) if has_integer => return Err(TypeCheckError::DuplicateLiteral(t.name().clone())),
            ExternLiteralSpecifier::Integer(_, _, _, _) => has_integer = true,
            ExternLiteralSpecifier::String if has_string => return Err(TypeCheckError::DuplicateLiteral(t.name().clone())),
            ExternLiteralSpecifier::String => has_string = true,
            ExternLiteralSpecifier::Sequence(_) if has_sequence => return Err(TypeCheckError::DuplicateLiteral(t.name().clone())),
            ExternLiteralSpecifier::Sequence(inner) => {
                has_sequence = true;
                tc.check_type(&BigUint::one(), inner)?;
            },
            ExternLiteralSpecifier::Case(name, params) => {
                if !literal_cases.insert(name) {
                    return Err(TypeCheckError::DuplicateLiteral(t.name().clone()));
                }

                for param in params {
                    tc.check_type(&BigUint::one(), param)?;
                }
            },
            ExternLiteralSpecifier::Record(_) if has_record => return Err(TypeCheckError::DuplicateLiteral(t.name().clone())),
            ExternLiteralSpecifier::Record(fields) => {
                has_record = true;
                for (_, field) in fields {
                    tc.check_type(&BigUint::one(), &field.field_type)?;
                }
            },
        }
    }

    Ok(())
}

fn type_check_constant<'model>(model: &'model Verilization, c: Named<'model, Constant>) -> Result<(), TypeCheckError> {
    let tc = TypeCheck {
        model: model,
        scope: c.scope(),
    };

    for ver in c.versions() {
        if !tc.check_value_type(&ver.version, c.value_type(), ver.value)? {
            return Err(TypeCheckError::ConstantHasIncorrectType(c.name().clone(), ver.version.clone()))
        }
    }

    Ok(())
}

pub fn type_check_verilization(model: &Verilization) -> Result<(), TypeCheckError> {
    
    for t in model.types() {
        match t {
            NamedTypeDefinition::StructType(t) | NamedTypeDefinition::EnumType(t) => type_check_versioned_type(model, t)?,
            NamedTypeDefinition::ExternType(t) => type_check_extern_type(model, t)?,
        }
    }

    for c in model.constants() {
        type_check_constant(model, c)?
    }

    Ok(())
}


