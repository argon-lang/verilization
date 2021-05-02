
export interface VerilizationModule {
    readonly memory: WebAssembly.Memory;

    verilization_mem_alloc(size: number): number;
    verilization_mem_free(size: number, ptr: number): void;

    verilization_parse(nfiles: number, files: number, result: number): void;
    verilization_destroy(verilization: number): void;

    verilization_generate(verilization: number, language: number, noptions: number, options: number, result: number): void;
}

export interface LangOption {
    readonly name: string;
    readonly value: string;
}
export type LangOptions = readonly LangOption[];

export interface OutputFileMap {
    [name: string]: Uint8Array | undefined;
}

export interface Verilization {
    generate(lang: string, options: LangOptions): OutputFileMap;
    close(): void;
}




const PTR_SIZE = 4;
const PtrArray = Uint32Array;




export class VerilizationCompiler {
    constructor(private module: VerilizationModule) {}




    private api_str_alloc(str: string): number {
        const data = new TextEncoder().encode(str);
        const strPtr = this.module.verilization_mem_alloc(PTR_SIZE + data.length);
        new PtrArray(this.module.memory.buffer, strPtr, 1)[0] = data.length;
        
        const fileStr = new Uint8Array(this.module.memory.buffer, strPtr + PTR_SIZE, data.length);
        for(let i = 0; i < data.length; ++i) {
            fileStr[i] = data[i];
        }

        return strPtr;
    }

    private api_str_free(ptr: number): void {
        this.module.verilization_mem_free(this.api_str_length(ptr) + PTR_SIZE, ptr);
    }


    private api_str_length(ptr: number): number {
        return new PtrArray(this.module.memory.buffer, ptr, 1)[0];
    }

    private api_str(ptr: number): string {
        const len = this.api_str_length(ptr);

        const textArr = new Uint8Array(this.module.memory.buffer, ptr + PTR_SIZE, len);
        return new TextDecoder().decode(textArr);
    }


    parse(files: readonly string[]): Verilization {
        const filesPtr = this.module.verilization_mem_alloc(files.length * PTR_SIZE);
        try {
            const filesArr = new PtrArray(this.module.memory.buffer, filesPtr, files.length);
            for(let i = 0; i < filesArr.length; ++i) {
                filesArr[i] = 0;
            }
            try {
                for(let i = 0; i < files.length; ++i) {
                    filesArr[i] = this.api_str_alloc(files[i]);
                }

                const resultPtr = this.module.verilization_mem_alloc(PTR_SIZE * 2);
                try {
                    this.module.verilization_parse(files.length, filesPtr, resultPtr);

                    const resultArr = new PtrArray(this.module.memory.buffer, resultPtr, 10);

                    if(resultArr[0] !== 0) {
                        const errorStr = this.api_str(resultArr[1]);
                        this.api_str_free(resultArr[1]);
                        throw new Error(errorStr);
                    }

                    return this.createImpl(resultArr[1]);
                }
                finally {
                    this.module.verilization_mem_free(PTR_SIZE * 2, resultPtr);
                }
            }
            finally {
                for(let i = 0; i < files.length; ++i) {
                    if(filesArr[i] !== 0) {
                        this.api_str_free(filesArr[i]);
                    }
                }
            }
        }
        finally {
            this.module.verilization_mem_free(files.length * PTR_SIZE, filesPtr);
        }
    }
    
    private createImpl(ptr: number): Verilization {
        const compiler = this;
        return {
            generate(lang: string, options: LangOptions): OutputFileMap {
                if(ptr === 0) {
                    throw new Error("Verilization object has been freed.");
                }
        
                const langPtr = compiler.api_str_alloc(lang);
                try {
                    const optionsPtr = compiler.module.verilization_mem_alloc(options.length * 2 * PTR_SIZE);
                    try {
                        const optionsArr = new PtrArray(compiler.module.memory.buffer, optionsPtr, options.length * 2);
                        for(let i = 0; i < options.length; ++i) {
                            optionsArr[2 * i] = compiler.api_str_alloc(options[i].name);
                            optionsArr[2 * i + 1] = compiler.api_str_alloc(options[i].value);
                        }
        
                        try {
                            const resultPtr = compiler.module.verilization_mem_alloc(2 * PTR_SIZE);
                            try {
                                compiler.module.verilization_generate(ptr, langPtr, options.length, optionsPtr, resultPtr);
        
                                const resultArr = new PtrArray(compiler.module.memory.buffer, resultPtr, 2);
        
                                if(resultArr[0] !== 0) {
                                    const errorStr = compiler.api_str(resultArr[1]);
                                    compiler.api_str_free(resultArr[1]);
                                    throw new Error(errorStr);
                                }
        
                                const mapPtr = resultArr[1];
                                const map: OutputFileMap = Object.create(null);
                                const numEntries = new PtrArray(compiler.module.memory.buffer, mapPtr, 1)[0];
                                try {
                                    const entryArr = new PtrArray(compiler.module.memory.buffer, mapPtr + PTR_SIZE, numEntries * 3);
                                    for(let i = 0; i < entryArr.length; i += 3) {
                                        const name = compiler.api_str(entryArr[i]);
                                        compiler.api_str_free(entryArr[i]);
        
                                        const len = entryArr[i + 1];
                                        const dataPtr = entryArr[i + 2];
                                        const data = new Uint8Array(compiler.module.memory.buffer.slice(dataPtr, dataPtr + len));
                                        compiler.module.verilization_mem_free(len, dataPtr);
        
                                        map[name] = data;
                                    }
                                }
                                finally {
                                    compiler.module.verilization_mem_free(mapPtr, (numEntries * 3 + 1) * PTR_SIZE);
                                }
        
                                return map;
                            }
                            finally {
                                compiler.module.verilization_mem_free(PTR_SIZE * 2, resultPtr);
                            }
                        }
                        finally {
                            for(let i = 0; i < optionsArr.length; ++i) {
                                compiler.api_str_free(optionsArr[i]);
                            }
                        }
                    }
                    finally {
                        compiler.module.verilization_mem_free(options.length * 2 * PTR_SIZE, optionsPtr);
                    }
                }
                finally {
                    compiler.api_str_free(langPtr);
                }
            },
        
            close(): void {
                if(ptr !== 0) {
                    compiler.module.verilization_destroy(ptr);
                    ptr = 0;
                }
            },
        }
    }
}


