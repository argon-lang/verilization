use crate::model::{Type, QualifiedName, Verilization, VersionedTypeDefinition, TypeDefinitionHandler, TypeDefinitionHandlerState};
use num_bigint::BigUint;
use std::collections::HashSet;

#[derive(Debug)]
pub enum TypeCheckError {
    TypeNotDefined(QualifiedName),
    TypeAddedInNewerVersion(QualifiedName, BigUint),
}

struct TypeCheck<'a> {
    verilization: &'a Verilization,
}


fn check_type(verilization: &Verilization, version: &BigUint, t: &Type) -> Result<(), TypeCheckError> {
    match t {
        Type::Defined(name) => {
            if !verilization.has_type(name) {
                Err(TypeCheckError::TypeNotDefined(name.clone()))
            }
            else if !verilization.has_type_in_version(name, version) {
                Err(TypeCheckError::TypeAddedInNewerVersion(name.clone(), version.clone()))
            }
            else {
                Ok(())
            }
        },
        Type::List(inner) => check_type(verilization, version, inner),
        Type::Option(inner) => check_type(verilization, version, inner),
        _ => Ok(())
    }
}

impl <'model, 'state> TypeDefinitionHandlerState<'model, 'state, TypeCheck<'model>, TypeCheckError> for TypeCheck<'model> where 'model : 'state {
    fn begin(outer: &'state mut TypeCheck<'model>, _type_name: &QualifiedName, _referenced_types: HashSet<&QualifiedName>) -> Result<Self, TypeCheckError> {
        Ok(TypeCheck {
            verilization: outer.verilization,
        })
    }

	fn versioned_type(&mut self, _explicit_version: bool, _type_name: &QualifiedName, version: &BigUint, type_definition: &VersionedTypeDefinition) -> Result<(), TypeCheckError> {

        for (_, field) in &type_definition.fields {
            check_type(self.verilization, version, &field.field_type)?
        }

        Ok(())
    }
	
    fn end(self, _struct_name: &QualifiedName) -> Result<(), TypeCheckError> {
        Ok(())
    }

}

impl <'model> TypeDefinitionHandler<'model, TypeCheckError> for TypeCheck<'model> {
    type StructHandlerState<'state> where 'model : 'state = TypeCheck<'model>;
    type EnumHandlerState<'state> where 'model : 'state = TypeCheck<'model>;
}


pub fn type_check_verilization(verilization: &Verilization) -> Result<(), TypeCheckError> {
    let mut tc = TypeCheck {
        verilization: verilization,
    };
    verilization.iter_types(&mut tc)
}


