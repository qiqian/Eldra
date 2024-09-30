package runtime
import kotlin.test.Test
import kotlin.test.assertTrue

class WasiTest {
    @Test
    fun mainTest() {
        val monotonicTime1 = wasiMonotonicTimeRuntime()
        val monotinicTime2 = wasiMonotonicTimeRuntime()
        assertTrue(monotonicTime1 <= monotinicTime2, "Wasi monotonic clock is not monotonic :(")
    }
}