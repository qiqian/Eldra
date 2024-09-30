package runtime

import kotlin.wasm.unsafe.MemoryAllocator
import kotlin.wasm.unsafe.UnsafeWasmMemoryApi
import kotlin.wasm.unsafe.withScopedMemoryAllocator

@WasmImport("env", "printf")
private external fun native_printf(scatterPtr: Int, errorPtr: Int): Int

@OptIn(UnsafeWasmMemoryApi::class)
internal fun wasiPrintImpl(
    allocator: MemoryAllocator,
    data: ByteArray?,
    newLine: Boolean,
    useErrorStream: Boolean
) {
    val dataSize = data?.size ?: 0
    val memorySize = dataSize + (if (newLine) 1 else 0) + 1

    val ptr = allocator.allocate(memorySize)
    if (data != null) {
        var currentPtr = ptr
        for (el in data) {
            currentPtr.storeByte(el)
            currentPtr += 1
        }
    }
    if (newLine) {
        (ptr + dataSize).storeShort(0x000A)
    } else {
        (ptr + dataSize).storeByte(0x00)
    }

    val rp0 = allocator.allocate(4)

    native_printf(ptr.address.toInt(), 0)
}

@OptIn(UnsafeWasmMemoryApi::class)
private fun printImpl(message: String?, useErrorStream: Boolean, newLine: Boolean) {
    withScopedMemoryAllocator { allocator ->
        wasiPrintImpl(
            allocator = allocator,
            data = message?.encodeToByteArray(),
            newLine = newLine,
            useErrorStream = useErrorStream,
        )
    }
}

/** Prints the line separator to the standard output stream. */
public fun wasi_println() {
    printImpl(null, useErrorStream = false, newLine = true)
}

/** Prints the given [message] and the line separator to the standard output stream. */
public fun wasi_println(message: Any?) {
    printImpl(message?.toString(), useErrorStream = false, newLine = true)
}

/** Prints the given [message] to the standard output stream. */
public fun wasi_print(message: Any?) {
    printImpl(message?.toString(), useErrorStream = false, newLine = false)
}