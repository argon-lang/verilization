use verilization_compiler::{
    model,
    lang::{
        language_registry_new,
        LanguageRegistry,
    },
};
use verilization_bindings_c_api_core::*;

#[no_mangle]
pub unsafe extern "C" fn verilization_generate(verilization: *const model::Verilization, language: *const APIString, noptions: usize, options: *const LanguageOption, result: *mut APIResult<OutputFileMap>) {
	let registry = language_registry_new()
		.add_language::<verilization_lang_typescript::TypeScriptLanguage>()
		.add_language::<verilization_lang_java::JavaLanguage>()
		.add_language::<verilization_lang_scala::ScalaLanguage>();

    verilization_generate_impl(verilization, language, noptions, options, result, &registry)
}