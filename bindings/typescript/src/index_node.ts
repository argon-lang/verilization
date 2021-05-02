
import {VerilizationCompiler, VerilizationModule, Verilization} from "./compiler.js";
export {Verilization, LangOption, LangOptions, OutputFileMap} from "./compiler.js";
import * as fs from "fs/promises";
import * as url from "url";


const moduleFile = new url.URL("./verilization_compiler.wasm", import.meta.url);

const moduleWasm = await WebAssembly.instantiate(await fs.readFile(moduleFile), {});

const compiler = new VerilizationCompiler(moduleWasm.instance.exports as unknown as VerilizationModule);


export function parse(files: readonly string[]): Verilization {
    return compiler.parse(files);
}



