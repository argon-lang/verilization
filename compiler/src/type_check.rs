use crate::model::{Type, QualifiedName, Verilization, VersionedTypeDefinition, TypeDefinitionHandler, TypeDefinitionHandlerState, Scope, ScopeLookup};
use num_bigint::BigUint;
use std::collections::HashSet;

#[derive(Debug)]
pub enum TypeCheckError {
    TypeNotDefined(QualifiedName),
    TypeAddedInNewerVersion(QualifiedName, BigUint),
    ArityMismatch(usize, usize),
}

struct TypeCheck<'a> {
    verilization: &'a Verilization,
}

struct TypeCheckType<'model, 'scope> {
    verilization: &'model Verilization,
    scope: &'scope Scope<'model>,
}

impl <'model, 'scope> TypeCheckType<'model, 'scope> {

    fn check_type(&self, version: &BigUint, t: &Type) -> Result<(), TypeCheckError> {
        match t {
            Type::Defined(name, args) => match self.scope.lookup(name.clone()) {
                ScopeLookup::NamedType(name) => {
                    let t = match self.verilization.get_type(&name) {
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
            Type::List(inner) => self.check_type(version, inner),
            Type::Option(inner) => self.check_type(version, inner),
            _ => Ok(())
        }
    }
    
}


impl <'model, 'state, 'scope> TypeDefinitionHandlerState<'model, 'state, 'scope, TypeCheck<'model>, TypeCheckError> for TypeCheckType<'model, 'scope> where 'model : 'state {
    fn begin(outer: &'state mut TypeCheck<'model>, _type_name: &QualifiedName, _type_params: &'model Vec<String>, scope: &'scope Scope<'model>,  _referenced_types: HashSet<&QualifiedName>) -> Result<Self, TypeCheckError> {
        Ok(TypeCheckType {
            verilization: outer.verilization,
            scope: scope,
        })
    }

	fn versioned_type(&mut self, _explicit_version: bool, version: &BigUint, type_definition: &VersionedTypeDefinition) -> Result<(), TypeCheckError> {

        for (_, field) in &type_definition.fields {
            self.check_type(version, &field.field_type)?
        }

        Ok(())
    }
	
    fn end(self) -> Result<(), TypeCheckError> {
        Ok(())
    }

}

impl <'model> TypeDefinitionHandler<'model, TypeCheckError> for TypeCheck<'model> {
    type StructHandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = TypeCheckType<'model, 'scope>;
    type EnumHandlerState<'state, 'scope> where 'model : 'scope, 'scope : 'state = TypeCheckType<'model, 'scope>;
}


pub fn type_check_verilization(verilization: &Verilization) -> Result<(), TypeCheckError> {
    let mut tc = TypeCheck {
        verilization: verilization,
    };
    verilization.iter_types(&mut tc)
}


