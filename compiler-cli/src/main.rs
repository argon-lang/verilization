use verilization_compiler::lang;
use lang::{LanguageRegistry, language_registry_new};
use verilization_compiler_cli_core::main_impl;

fn main() {
	let registry = language_registry_new()
		.add_language::<verilization_lang_typescript::TypeScriptLanguage>()
		.add_language::<verilization_lang_java::JavaLanguage>()
		.add_language::<verilization_lang_scala::ScalaLanguage>();

	main_impl(&registry)
}
