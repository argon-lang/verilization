import * as verilization from "./verilization_compiler.wasm";

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

function api_str_alloc(str: string): number {
    const data = new TextEncoder().encode(str);
    const strPtr = verilization.verilization_mem_alloc(PTR_SIZE + data.length);
    new PtrArray(verilization.memory.buffer, strPtr, 1)[0] = data.length;
    
    const fileStr = new Uint8Array(verilization.memory.buffer, strPtr + PTR_SIZE, data.length);
    for(let i = 0; i < data.length; ++i) {
        fileStr[i] = data[i];
    }

    return strPtr;
}

function api_str_free(ptr: number): void {
    verilization.verilization_mem_free(api_str_length(ptr) + PTR_SIZE, ptr);
}


function api_str_length(ptr: number): number {
    return new PtrArray(verilization.memory.buffer, ptr, 1)[0];
}

function api_str(ptr: number): string {
    const len = api_str_length(ptr);

    const textArr = new Uint8Array(verilization.memory.buffer, ptr + PTR_SIZE, len);
    return new TextDecoder().decode(textArr);
}




class VerilizationImpl implements Verilization {
    constructor(private ptr: number) {}

    generate(lang: string, options: LangOptions): OutputFileMap {
        if(this.ptr === 0) {
            throw new Error("Verilization object has been freed.");
        }

        const langPtr = api_str_alloc(lang);
        try {
            const optionsPtr = verilization.verilization_mem_alloc(options.length * 2 * PTR_SIZE);
            try {
                const optionsArr = new PtrArray(verilization.memory.buffer, optionsPtr, options.length * 2);
                for(let i = 0; i < options.length; ++i) {
                    optionsArr[2 * i] = api_str_alloc(options[i].name);
                    optionsArr[2 * i + 1] = api_str_alloc(options[i].value);
                }

                try {
                    const resultPtr = verilization.verilization_mem_alloc(2 * PTR_SIZE);
                    try {
                        verilization.verilization_generate(this.ptr, langPtr, options.length, optionsPtr, resultPtr);

                        const resultArr = new PtrArray(verilization.memory.buffer, resultPtr, 2);

                        if(resultArr[0] !== 0) {
                            const errorStr = api_str(resultArr[1]);
                            api_str_free(resultArr[1]);
                            throw new Error(errorStr);
                        }

                        const mapPtr = resultArr[1];
                        const map: OutputFileMap = Object.create(null);
                        const numEntries = new PtrArray(verilization.memory.buffer, mapPtr, 1)[0];
                        try {
                            const entryArr = new PtrArray(verilization.memory.buffer, mapPtr + PTR_SIZE, numEntries * 3);
                            for(let i = 0; i < entryArr.length; i += 3) {
                                const name = api_str(entryArr[i]);
                                api_str_free(entryArr[i]);

                                const len = entryArr[i + 1];
                                const dataPtr = entryArr[i + 2];
                                const data = new Uint8Array(verilization.memory.buffer.slice(dataPtr, dataPtr + len));
                                verilization.verilization_mem_free(len, dataPtr);

                                map[name] = data;
                            }
                        }
                        finally {
                            verilization.verilization_mem_free(mapPtr, (numEntries * 3 + 1) * PTR_SIZE);
                        }

                        return map;
                    }
                    finally {
                        verilization.verilization_mem_free(PTR_SIZE * 2, resultPtr);
                    }
                }
                finally {
                    for(let i = 0; i < optionsArr.length; ++i) {
                        api_str_free(optionsArr[i]);
                    }
                }
            }
            finally {
                verilization.verilization_mem_free(options.length * 2 * PTR_SIZE, optionsPtr);
            }
        }
        finally {
            api_str_free(langPtr);
        }
    }

    close(): void {
        if(this.ptr !== 0) {
            verilization.verilization_destroy(this.ptr);
            this.ptr = 0;
        }
    }

    
}


export namespace Verilization {
    export function parse(files: readonly string[]): Verilization {
        const filesPtr = verilization.verilization_mem_alloc(files.length * PTR_SIZE);
        try {
            const filesArr = new PtrArray(verilization.memory.buffer, filesPtr, files.length);
            for(let i = 0; i < filesArr.length; ++i) {
                filesArr[i] = 0;
            }
            try {
                for(let i = 0; i < files.length; ++i) {
                    filesArr[i] = api_str_alloc(files[i]);
                }

                const resultPtr = verilization.verilization_mem_alloc(PTR_SIZE * 2);
                try {
                    verilization.verilization_parse(files.length, filesPtr, resultPtr);

                    const resultArr = new PtrArray(verilization.memory.buffer, resultPtr, 10);

                    if(resultArr[0] !== 0) {
                        const errorStr = api_str(resultArr[1]);
                        api_str_free(resultArr[1]);
                        throw new Error(errorStr);
                    }

                    return new VerilizationImpl(resultArr[1]);
                }
                finally {
                    verilization.verilization_mem_free(PTR_SIZE * 2, resultPtr);
                }
            }
            finally {
                for(let i = 0; i < files.length; ++i) {
                    if(filesArr[i] !== 0) {
                        api_str_free(filesArr[i]);
                    }
                }
            }
        }
        finally {
            verilization.verilization_mem_free(files.length * PTR_SIZE, filesPtr);
        }
    }
}


