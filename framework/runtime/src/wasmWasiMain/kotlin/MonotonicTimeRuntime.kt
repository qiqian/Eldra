package runtime
import kotlin.wasm.WasmImport
import kotlin.wasm.unsafe.Pointer
import kotlin.wasm.unsafe.UnsafeWasmMemoryApi
import kotlin.wasm.unsafe.withScopedMemoryAllocator

@WasmImport("engine", "foo_test")
private external fun foo_test(a: Int, b: Int): Int

@WasmExport
fun wasm_main(a:Int, b:Int):Long {
    // this works
    //return wasiRealTimeRuntime()

    // println has try/catch/throw, not working
    wasi_println("Current 'realtime' timestamp is: ${wasiRealTimeRuntime()}")
    wasi_println("Helloe from Kotlin via WASI")
    wasi_println("Current 'monotonic' timestamp is: ${wasiMonotonicTimeRuntime()}")
    var ret = foo_test(a, b);
    wasi_println("foo_test: ${ret}")
    return ret.toLong();
}

@WasmImport("wasi_snapshot_preview1", "clock_time_get")
private external fun wasiRawClockTimeGet(clockId: Int, precision: Long, resultPtr: Int): Int

private const val REALTIME = 0
private const val MONOTONIC = 1

@OptIn(UnsafeWasmMemoryApi::class)
fun wasiGetTime(clockId: Int): Long = withScopedMemoryAllocator { allocator ->
    val rp0 = allocator.allocate(8)
    val ret = wasiRawClockTimeGet(
        clockId = clockId,
        precision = 2,
        resultPtr = rp0.address.toInt()
    )
    check(ret == 0) {
        "Invalid WASI return code $ret"
    }
    (Pointer(rp0.address.toInt().toUInt())).loadLong()
}

fun wasiRealTimeRuntime(): Long = wasiGetTime(REALTIME)

@WasmExport
fun wasiMonotonicTimeRuntime(): Long = wasiGetTime(MONOTONIC)

// We need it to run WasmEdge with the _initialize function
@WasmExport
fun dummyRuntime() {}