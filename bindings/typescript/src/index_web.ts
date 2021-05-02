
import {VerilizationCompiler, VerilizationModule, Verilization} from "./compiler.js";
export {Verilization, LangOption, LangOptions, OutputFileMap} from "./compiler.js";

const moduleWasm = await WebAssembly.instantiateStreaming(fetch("verilization_compiler.wasm"));

const compiler = new VerilizationCompiler(moduleWasm.instance.exports as unknown as VerilizationModule);


export function parse(files: readonly string[]): Verilization {
    return compiler.parse(files);
}



