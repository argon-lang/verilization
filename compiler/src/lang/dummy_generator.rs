use crate::model;
use model::Named;
use super::generator::*;
use super::GeneratorError;
use std::marker::PhantomData;

struct DummyConstGenerator<'model, Lang> {
    constant: Named<'model, model::Constant>,
    model: &'model model::Verilization,
    scope: model::Scope<'model>,
    dummy_lang: PhantomData<Lang>,
}

impl <'model, Lang: GeneratorNameMapping> Generator<'model> for DummyConstGenerator<'model, Lang> {
    type Lang = Lang;

	fn model(&self) -> &'model model::Verilization {
        self.model
    }

	fn scope(&self) -> &model::Scope<'model> {
        &self.scope
    }
}

impl <'model, Lang: GeneratorNameMapping> ConstGenerator<'model> for DummyConstGenerator<'model, Lang> {
	fn constant(&self) -> Named<'model, model::Constant> {
        self.constant
    }

	fn write_header(&mut self) -> Result<(), GeneratorError> {
        Ok(())
    }

	fn write_constant(&mut self, _version_name: String, _t: LangType<'model>, _value: LangExpr<'model>) -> Result<(), GeneratorError> {
        Ok(())
    }

	fn write_footer(&mut self) -> Result<(), GeneratorError> {
        Ok(())
    }

}
