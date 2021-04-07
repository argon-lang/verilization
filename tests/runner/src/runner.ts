import {spawn, SpawnOptions} from "child_process";
import {LanguageHandler} from "./LanguageHandler.js";
import {TypeScriptHandler} from "./TypeScriptHandler.js";


const languages: ReadonlyArray<LanguageHandler> = [
    new TypeScriptHandler({
        packageMapping: [
            [ "struct.versions", "struct/versions" ],
            [ "enum.versions", "enum/versions" ],
        ],
    }),
    {
        name: "java",
        options: [
            "-o:out_dir", "tests/java/gen/",
            "-o:pkg:struct.versions", "struct.versions",
            "-o:pkg:enum.versions", "enum_.versions",
        ],
    },
];

const files: ReadonlyArray<string> = [
    "struct_versions",
    "enum_versions",
];


for(const file of files) {
    for(const lang of languages) {
        await runCommand("../../", "cargo", "run", "-q", "--", "generate", lang.name, "-i", `tests/verilization/${file}.verilization`, ...lang.generateCommandOptions);
    }
}



function runCommand(workingDir: string, cmd: string, ...args: string[]) {
	return runCommandOptions({
		cwd: workingDir,
		stdio: "inherit",
	}, cmd, ...args);
}

function runCommandOptions(options: SpawnOptions, cmd: string, ...args: string[]): Promise<void> {
	return new Promise((resolve, reject) => {
		const process = spawn(cmd, args, options);

		process.on("close", code => {
			if(code !== 0) {
				reject(new Error(`Process exited with error code ${code}.`));
			}
			else {
				resolve();
			}
		});
	});
}

