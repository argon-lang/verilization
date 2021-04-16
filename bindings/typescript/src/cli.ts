import {Verilization, OutputFileMap, LangOptions, LangOption} from "./index.js";
import * as fs from "fs/promises";
import * as path from "path";

function command_version(): void {
    console.log("TODO: Version");
}

function command_help(): void {
    console.log("TODO: Help");
}

async function command_generate(lang: string, inputFiles: readonly string[], options: LangOptions): Promise<void> {
    const file_content = await Promise.all(inputFiles.map(filename =>
        fs.readFile(filename, { encoding: "utf-8" })
    ));
    const model = Verilization.parse(file_content);
    let output: OutputFileMap;
    try {
        output = model.generate(lang, options);
    }
    finally {
        model.close();
    }
    
    for(let filename in output) {
        if(!Object.prototype.hasOwnProperty.call(output, filename)) continue;
        const data = output[filename];
        if(data === undefined) continue;

        const dir = path.dirname(filename);
        
        await fs.mkdir(dir, { recursive: true });
        await fs.writeFile(filename, data);
    }
}

function parse_generate_command(args: Iterator<string>, lang: string): Promise<void> {
    const inputFiles: string[] = [];
    const options: LangOption[] = [];

    while(true) {
        const argItem = args.next();
        if(argItem.done) break;
        const arg = argItem.value;
        
        if(arg === "-i") {
            const inputFile = args.next();
            if(inputFile.done) {
                throw new Error("Missing value for input file");
            }

            inputFiles.push(inputFile.value);
        }
        else if(arg.startsWith("-o:")) {
            const optionName = arg.substr(3);

            const optionValue = args.next();
            if(optionValue.done) {
                throw new Error(`Missing value for option ${optionName}`);
            }
            
            options.push({ name: optionName, value: optionValue.value });
        }
        else {
            throw new Error(`Unknown argument: ${arg}`);
        }
    }

    return command_generate(lang, inputFiles, options);
}

async function parse_args(args: Iterator<string>): Promise<void> {
    while(true) {
        const argItem = args.next();
        if(argItem.done) break;
        const arg = argItem.value;

        switch(arg) {
            case "version":
            case "--version":
            case "-v":
                return command_version();

            case "help":
            case "--help":
            case "-h":
                return command_help();

            case "generate":
            {
                const langArg = args.next();
                if(langArg.done) {
                    throw new Error("Language not specified");
                }

                return await parse_generate_command(args, langArg.value);
            }

            default:
                throw new Error(`Unexpected argument: ${arg}`);
        }
    }

    throw new Error("No command specified");
}

try {
    await parse_args(process.argv.slice(2)[Symbol.iterator]());
}
catch(e) {
    process.exitCode = 1;
}
