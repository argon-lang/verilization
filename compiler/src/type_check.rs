use crate::model::{
    Type,
    QualifiedName,
    Verilization,
    NamedTypeDefinition,
    Scope,
    ScopeLookup,
};
use num_bigint::BigUint;

#[derive(Debug)]
pub enum TypeCheckError {
    TypeNotDefined(QualifiedName),
    TypeAddedInNewerVersion(QualifiedName, BigUint),
    ArityMismatch(usize, usize),
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
    
}


pub fn type_check_verilization(model: &Verilization) -> Result<(), TypeCheckError> {
    
    for t in model.types() {
        let t = match t {
            NamedTypeDefinition::StructType(t) => t,
            NamedTypeDefinition::EnumType(t) => t,
            NamedTypeDefinition::ExternType(_) => continue,
        };

        let tc = TypeCheck {
            model: model,
            scope: t.scope(),
        };

        for ver in t.versions() {
            for (_, field) in &ver.ver_type.fields {
                tc.check_type(&ver.version, &field.field_type)?;
            }
        }
    }

    Ok(())
}


