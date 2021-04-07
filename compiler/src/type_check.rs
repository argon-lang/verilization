use crate::model::{Type, QualifiedName, Verilization, VersionedTypeDefinition, TypeDefinitionHandler};
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

impl <'model, 'state> TypeDefinitionHandler<'model, 'state, TypeCheckError> for TypeCheck<'model> {
    type StructHandlerState = TypeCheck<'model>;
	
    fn begin_struct(&'state mut self, _struct_name: &QualifiedName, _referenced_types: HashSet<&QualifiedName>) -> Result<Self::StructHandlerState, TypeCheckError> {
        Ok(TypeCheck {
            verilization: self.verilization,
        })
    }

	fn versioned_struct(state: &mut Self::StructHandlerState, _explicit_version: bool, _struct_name: &QualifiedName, version: &BigUint, type_definition: &VersionedTypeDefinition) -> Result<(), TypeCheckError> {

        for (_, field) in &type_definition.fields {
            check_type(state.verilization, version, &field.field_type)?
        }

        Ok(())
    }
	
    fn end_struct(_state: Self::StructHandlerState, _struct_name: &QualifiedName) -> Result<(), TypeCheckError> {
        Ok(())
    }
	

    type EnumHandlerState = TypeCheck<'model>;

    fn begin_enum(&'state mut self, _enum_name: &QualifiedName, _referenced_types: HashSet<&QualifiedName>) -> Result<Self::StructHandlerState, TypeCheckError> {
        Ok(TypeCheck {
            verilization: self.verilization,
        })
    }

	fn versioned_enum(state: &mut Self::StructHandlerState, _explicit_version: bool, _enum_name: &QualifiedName, version: &BigUint, type_definition: &VersionedTypeDefinition) -> Result<(), TypeCheckError> {

        for (_, field) in &type_definition.fields {
            check_type(state.verilization, version, &field.field_type)?
        }

        Ok(())
    }
	
    fn end_enum(_state: Self::StructHandlerState, _enum_name: &QualifiedName) -> Result<(), TypeCheckError> {
        Ok(())
    }

}


pub fn type_check_verilization(verilization: &Verilization) -> Result<(), TypeCheckError> {
    let mut tc = TypeCheck {
        verilization: verilization,
    };
    verilization.iter_types(&mut tc)
}


