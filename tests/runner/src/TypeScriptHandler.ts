import {LanguageHandler} from "./LanguageHandler.js";

export interface TypeScriptHandlerOptions {
    packageMapping: [string, string][],
}

export class TypeScriptHandler implements LanguageHandler {
    
    constructor(options: TypeScriptHandlerOptions) {
        this.options = options;
    }

    private readonly options: TypeScriptHandlerOptions;

    get name(): string {
        return "typescript";
    }

    get generateCommandOptions(): readonly string[] {
        return [
            "-o:out_dir", "tests/typescript/src/gen/",
            ...this.options.packageMapping.flatMap(([verPkg, tsDir]) => [`-o:pkg:${verPkg}`, tsDir])
        ];
    }
    
}
