#include <iostream>
#include <fstream>
#include <vector>
#include <stdint.h>
#include "wasm_export.h"

static
int foo_native(wasm_exec_env_t exec_env , int a, int b)
{
    printf("a = %d\n", a);
    printf("b = %d\n", b);
    return a+b;
}

int main(int argc, const char* argv[])
{
    /* initialize the wasm runtime by default configurations */
    wasm_runtime_init();

    /* add line below if we want to export native functions to WASM app */
    static NativeSymbol native_symbols[] =
    {
        {
            "foo_test", 		// the name of WASM function name
             foo_native, // the native function pointer
            "(ii)i"		// the function prototype signature
        },
    };
    constexpr int n_native_symbols = sizeof(native_symbols) / sizeof(NativeSymbol);
    if (!wasm_runtime_register_natives("engine",
                                       native_symbols,
                                       n_native_symbols))
        return -1;

    /* read WASM file into a memory buffer */
    char *buffer;
    uint32_t size;
    /*Open the stream in binary mode.*/
    std::ifstream bin_file(argv[1], std::ios::binary);

    if (!bin_file.good())
        return -2;

    char error_buf[128];
    wasm_module_t module;
    std::vector<uint8_t> v_buf((std::istreambuf_iterator<char>(bin_file)), (std::istreambuf_iterator<char>()));
    {
        /*Read Binary data using streambuffer iterators.*/
        bin_file.close();

        /* parse the WASM file from buffer and create a WASM module */
        module = wasm_runtime_load((uint8_t*)v_buf.data(), v_buf.size(), error_buf, sizeof(error_buf));
    }

    /* create an instance of the WASM module (WASM linear memory is ready) */
    uint32_t stack_size = 8092, heap_size = 8092 * 1024;
    wasm_module_inst_t module_inst = wasm_runtime_instantiate(module, stack_size, heap_size, error_buf, sizeof(error_buf));

    /* lookup a WASM function by its name
     The function signature can NULL here */
    wasm_function_inst_t func = wasm_runtime_lookup_function(module_inst, "wasm_main");
    if (!func)
        return -3;

    /* creat an execution environment to execute the WASM functions */
    wasm_exec_env_t exec_env = wasm_runtime_create_exec_env(module_inst, stack_size);

    {
        uint32_t wasm_argv[3];
        /* arguments are always transferred in 32-bit element */
        wasm_argv[0] = 8;
        wasm_argv[1] = 1;

        /* call the WASM function */
        printf("call wasm_main\n");
        if (wasm_runtime_call_wasm(exec_env, func, 2, wasm_argv) ) {
            /* the return value is stored in argv[0] */
            printf("wasm_main function return: %d\n", wasm_argv[0]);
        }
        else {
            /* exception is thrown if call fails */
            printf("%s\n", wasm_runtime_get_exception(module_inst));
        }
    }

    return 0;
}
