package editor
import kotlin.test.Test
import kotlin.test.assertTrue
import runtime.wasiMonotonicTimeRuntime

class WasiTestEditor {
    @Test
    fun mainTest() {
        val monotonicTime1 = wasiMonotonicTimeEditor()
        val monotinicTime2 = wasiMonotonicTimeRuntime()
        assertTrue(monotonicTime1 <= monotinicTime2, "Wasi monotonic clock is not monotonic :(")
    }
}