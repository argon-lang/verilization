use crate::model::*;
use num_bigint::BigUint;
use num_traits::One;
use std::collections::HashSet;

#[derive(Debug)]
pub enum TypeCheckError {
    TypeNotDefined(QualifiedName),
    TypeAddedInNewerVersion(QualifiedName, BigUint),
    CouldNotFindLastVersion(QualifiedName),
    ArityMismatch(usize, usize),
    TypeNotFinal(QualifiedName),
    DuplicateLiteral(QualifiedName),
}

struct TypeCheck<'model> {
    model: &'model Verilization,
    scope: Scope<'model>,
}

impl <'model> TypeCheck<'model> {

    fn check_type(&self, version: &BigUint, t: &Type) -> Result<(), TypeCheckError> {
        match t {
            Type::Defined(name, args) => match self.scope.lookup(name.clone()) {
                ScopeLookup::NamedType(name) => {
                    let t = match self.model.get_type(&name) {
                        Some(t) => t,
                        None => return Err(TypeCheckError::TypeNotDefined(name)),
                    };
    
                    if !t.has_version(version) {
                        return Err(TypeCheckError::TypeAddedInNewerVersion(name, version.clone()));
                    }
        
                    let arity = t.arity();
                    if arity != args.len() {
                        return Err(TypeCheckError::ArityMismatch(arity, args.len()));
                    }
        
                    for arg in args {
                        self.check_type(version, arg)?;
                    }
    
                    Ok(())
                },
                ScopeLookup::TypeParameter(_) => {
                    if args.len() != 0 {
                        return Err(TypeCheckError::ArityMismatch(0, args.len()));
                    }

                    Ok(())
                }
            },
        }
    }

    fn check_is_final(&self, t: &Type, version: &BigUint) -> Result<bool, TypeCheckError> {
        Ok(match t {
            Type::Defined(name, args) => {
                match self.scope.lookup(name.clone()) {
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
    
                        for arg in args {
                            if !self.check_is_final(arg, version)? {
                                return Ok(false);
                            }
                        }
    
                        true
                    },
                    ScopeLookup::TypeParameter(_) => true,
                }
            },
        })
    }
    
}

fn type_check_versioned_type<'model>(model: &'model Verilization, t: Named<'model, VersionedTypeDefinitionData>) -> Result<(), TypeCheckError> {
    let tc = TypeCheck {
        model: model,
        scope: t.scope(),
    };

    for ver in t.versions() {
        for (_, field) in &ver.ver_type.fields {
            tc.check_type(&ver.version, &field.field_type)?;
        }
    }

    if t.is_final() {
        if let Some(last_ver) = t.versions().last() {
            let args = t.type_params().iter().map(|param| Type::Defined(QualifiedName::from_parts(&[], &param), vec!())).collect::<Vec<_>>();

            for (_, field) in &last_ver.ver_type.fields {
                if !tc.check_is_final(&field.field_type, &last_ver.version)? {
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
            ExternLiteralSpecifier::Sequence(inner) if has_sequence => return Err(TypeCheckError::DuplicateLiteral(t.name().clone())),
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
            ExternLiteralSpecifier::Record(fields) => {
                for (_, field) in fields {
                    tc.check_type(&BigUint::one(), &field.field_type)?;
                }
            },
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

    Ok(())
}


