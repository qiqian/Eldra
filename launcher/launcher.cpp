#include <iostream>
#include <stdint.h>
#include "wasm_export.h"

int main()
{
    char *buffer, error_buf[128];
    wasm_module_t module;
    wasm_module_inst_t module_inst;
    wasm_function_inst_t func;
    wasm_exec_env_t exec_env;
    uint32_t size, stack_size = 8092, heap_size = 8092;

    /* initialize the wasm runtime by default configurations */
    wasm_runtime_init();

    /* read WASM file into a memory buffer */
    //buffer = read_wasm_binary_to_buffer(0, &size);

    /* add line below if we want to export native functions to WASM app */
    //wasm_runtime_register_natives(...);

    /* parse the WASM file from buffer and create a WASM module */
    module = wasm_runtime_load((uint8_t*)buffer, size, error_buf, sizeof(error_buf));

    /* create an instance of the WASM module (WASM linear memory is ready) */
    module_inst = wasm_runtime_instantiate(module, stack_size, heap_size,
                                           error_buf, sizeof(error_buf));

    return 0;
}
