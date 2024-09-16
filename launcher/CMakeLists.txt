cmake_minimum_required(VERSION 3.29)
project(launcher)

set (SOURCE_DEBUGGING 0)

set (CMAKE_WARN_DEPRECATED OFF CACHE BOOL "" FORCE)
set (SUBMODULE_DIR ${CMAKE_CURRENT_SOURCE_DIR}/../submodule)

set (WAMR_BUILD_PLATFORM "windows")

set (WAMR_ROOT_DIR ${SUBMODULE_DIR}/wamr)
set (WAMR_BUILD_GC 1)
set (WAMR_BUILD_MEMORY64 0)
set (WAMR_BUILD_FAST_INTERP 1)
set (WAMR_BUILD_EXCE_HANDLING 1)
set (WAMR_BUILD_MULTI_MODULE 1)
set (WAMR_BUILD_THREAD_MGR 1)
set (WAMR_BUILD_LIBC_BUILTIN 1)
set (WAMR_BUILD_LIBC_WASI 1)
set (WAMR_BUILD_LIBC_UVWASI 0)
set (WAMR_BUILD_LIB_WASI_THREADS 1)
set (WAMR_BUILD_LIB_PTHREAD 1)
set (WAMR_BUILD_LIB_PTHREAD_SEMAPHORE 1)
set (WAMR_BUILD_TAIL_CALL 1)
set (WAMR_BUILD_CUSTOM_NAME_SECTION 1)
set (WAMR_BUILD_DUMP_CALL_STACK 1)
set (WAMR_BUILD_GLOBAL_HEAP_POOL 1)
set (WAMR_BUILD_REF_TYPES 1)
set (WAMR_BUILD_LOAD_CUSTOM_SECTION 1)
set (WAMR_CONFIGURABLE_BOUNDS_CHECKS 1)

# set 1 to enable debug
set (WAMR_BUILD_DEBUG_INTERP ${SOURCE_DEBUGGING})
set (WAMR_DISABLE_HW_BOUND_CHECK ${SOURCE_DEBUGGING})
set (WAMR_DISABLE_STACK_HW_BOUND_CHECK ${SOURCE_DEBUGGING})

# Reset default linker flags
set (CMAKE_SHARED_LIBRARY_LINK_C_FLAGS "")
set (CMAKE_SHARED_LIBRARY_LINK_CXX_FLAGS "")

add_definitions(-DCOMPILING_WASM_RUNTIME_API=1)

# Set WAMR_BUILD_TARGET, currently values supported:
# "X86_64", "AMD_64", "X86_32", "AARCH64[sub]", "ARM[sub]", "THUMB[sub]", "MIPS", "XTENSA"
if (NOT DEFINED WAMR_BUILD_TARGET)
    if (CMAKE_SIZEOF_VOID_P EQUAL 8)
        # Build as X86_64 by default in 64-bit platform
        set (WAMR_BUILD_TARGET "X86_64")
    elseif (CMAKE_SIZEOF_VOID_P EQUAL 4)
        # Build as X86_32 by default in 32-bit platform
        set (WAMR_BUILD_TARGET "X86_32")
    else ()
        message(FATAL_ERROR "Unsupported build target platform!")
    endif ()
endif ()

if (NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Release)
endif ()

if (NOT DEFINED WAMR_BUILD_INTERP)
    # Enable Interpreter by default
    set (WAMR_BUILD_INTERP 1)
endif ()

if (NOT DEFINED WAMR_BUILD_AOT)
    # Enable AOT by default.
    set (WAMR_BUILD_AOT 1)
endif ()

if (NOT DEFINED WAMR_BUILD_JIT)
    # Disable JIT by default.
    set (WAMR_BUILD_JIT 0)
endif ()

if (NOT DEFINED WAMR_BUILD_LIBC_BUILTIN)
    # Enable libc builtin support by default
    set (WAMR_BUILD_LIBC_BUILTIN 1)
endif ()

if (NOT DEFINED WAMR_BUILD_LIBC_UVWASI)
    # Enable libc uvwasi support by default
    set (WAMR_BUILD_LIBC_UVWASI 1)
endif ()

if (NOT DEFINED WAMR_BUILD_FAST_INTERP)
    # Enable fast interpreter
    set (WAMR_BUILD_FAST_INTERP 1)
endif ()

if (NOT DEFINED WAMR_BUILD_MULTI_MODULE)
    # Enable multiple modules
    set (WAMR_BUILD_MULTI_MODULE 0)
endif ()

if (NOT DEFINED WAMR_BUILD_LIB_PTHREAD)
    # Disable pthread library by default
    set (WAMR_BUILD_LIB_PTHREAD 0)
endif ()

if (NOT DEFINED WAMR_BUILD_MINI_LOADER)
    # Disable wasm mini loader by default
    set (WAMR_BUILD_MINI_LOADER 0)
endif ()

if (NOT DEFINED WAMR_BUILD_SIMD)
    # Enable SIMD by default
    set (WAMR_BUILD_SIMD 1)
endif ()

if (NOT DEFINED WAMR_BUILD_DEBUG_INTERP)
    # Disable Debug feature by default
    set (WAMR_BUILD_DEBUG_INTERP 0)
endif ()

if (WAMR_BUILD_DEBUG_INTERP EQUAL 1)
    set (WAMR_BUILD_FAST_INTERP 0)
    set (WAMR_BUILD_MINI_LOADER 0)
    set (WAMR_BUILD_SIMD 0)
endif ()

if (WAMR_BUILD_LIBC_WASI EQUAL 1)
    set (CMAKE_C_STANDARD 11)
    if (MSVC)
        add_compile_options(/experimental:c11atomics)
    endif()
else()
    set (CMAKE_C_STANDARD 99)
endif()

include (${WAMR_ROOT_DIR}/build-scripts/runtime_lib.cmake)
add_library(vmlib ${WAMR_RUNTIME_LIB_SOURCE})

#set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -DWIN32_LEAN_AND_MEAN")
if (NOT MINGW)
    set (CMAKE_EXE_LINKER_FLAGS "${CMAKE_EXE_LINKER_FLAGS} /SAFESEH:NO")
    set (CMAKE_SHARED_LINKER_FLAGS "${CMAKE_SHARED_LINKER_FLAGS} /SAFESEH:NO")
endif ()

# set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra -Wformat -Wformat-security")
# set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wconversion -Wsign-conversion")

if (WAMR_BUILD_TARGET MATCHES "X86_.*" OR WAMR_BUILD_TARGET STREQUAL "AMD_64")
    if (NOT (CMAKE_C_COMPILER MATCHES ".*clang.*" OR CMAKE_C_COMPILER_ID MATCHES ".*Clang" OR MSVC))
        set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -mindirect-branch-register")
    endif ()
endif ()

# The following flags are to enhance security, but it may impact performance,
# we disable them by default.
#if (WAMR_BUILD_TARGET MATCHES "X86_.*" OR WAMR_BUILD_TARGET STREQUAL "AMD_64")
#  set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -ftrapv -D_FORTIFY_SOURCE=2")
#endif ()
#set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fstack-protector-strong --param ssp-buffer-size=4")
#set (CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wl,-z,noexecstack,-z,relro,-z,now")

include (${SHARED_DIR}/utils/uncommon/shared_uncommon.cmake)

add_executable (iwasm ${WAMR_ROOT_DIR}/product-mini/platforms/${WAMR_BUILD_PLATFORM}/main.c ${UNCOMMON_SHARED_SOURCE})

install (TARGETS iwasm DESTINATION bin)

target_link_libraries (iwasm vmlib ${LLVM_AVAILABLE_LIBS} ${UV_A_LIBS})

if (MINGW)
    target_link_libraries (iwasm ws2_32)
endif ()

add_library (libiwasm SHARED ${WAMR_RUNTIME_LIB_SOURCE})

install (TARGETS libiwasm DESTINATION lib)

set_target_properties (libiwasm PROPERTIES OUTPUT_NAME libiwasm)

target_link_libraries (libiwasm ${LLVM_AVAILABLE_LIBS} ${UV_A_LIBS})

if (MINGW)
    target_link_libraries (libiwasm ws2_32)
endif ()

if (WIN32)
    target_link_libraries(libiwasm ntdll)

    target_link_libraries(iwasm ntdll)
endif()

add_executable(launcher launcher.cpp)
if (WIN32)
    add_subdirectory(${SUBMODULE_DIR}/pthread-win32 submodule/pthread-win32)
    set_property(SOURCE ${WAMR_ROOT_DIR}/core/iwasm/libraries/lib-pthread/lib_pthread_wrapper.c
            APPEND PROPERTY "COMPILE_OPTIONS"
            "/Dos_sem_open=sem_open" "/Dos_sem_close=sem_close" "/Dos_sem_trywait=sem_trywait"
            "/Dos_sem_post=sem_post" "/Dos_sem_getvalue=sem_getvalue" "/Dos_sem_unlink=sem_unlink")
    target_include_directories(launcher BEFORE PUBLIC ${SUBMODULE_DIR}/pthread-win32)
    target_link_libraries (launcher ntdll pthreadVC3)
endif()
target_link_libraries (launcher vmlib ${LLVM_AVAILABLE_LIBS} ${UV_A_LIBS})
install (TARGETS launcher DESTINATION bin)