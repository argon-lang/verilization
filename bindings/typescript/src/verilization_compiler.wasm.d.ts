
export declare const memory: WebAssembly.Memory;

export declare function verilization_mem_alloc(size: number): number;
export declare function verilization_mem_free(size: number, ptr: number): void;

export declare function verilization_parse(nfiles: number, files: number, result: number): void;
export declare function verilization_destroy(verilization: number): void;

export declare function verilization_generate(verilization: number, language: number, noptions: number, options: number, result: number): void;
