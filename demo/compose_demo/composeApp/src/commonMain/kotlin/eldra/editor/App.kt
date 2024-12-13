package eldra.editor

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.Button
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.geometry.isFinite
import androidx.compose.ui.graphics.*
import androidx.compose.ui.unit.dp
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.ui.tooling.preview.Preview

import kotlin.math.abs

import compose_demo.composeapp.generated.resources.Res
import compose_demo.composeapp.generated.resources.compose_multiplatform
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.sync.Semaphore
import kotlinx.coroutines.withContext
import kotlinx.atomicfu.*

@Immutable
class TexBrush internal constructor(
    private val colors: List<Color>,
    private val stops: List<Float>? = null,
    private val start: Offset,
    private val end: Offset,
    private val tileMode: TileMode = TileMode.Clamp
) : ShaderBrush() {

    override val intrinsicSize: Size
        get() =
            Size(
                if (start.x.isFinite() && end.x.isFinite()) abs(start.x - end.x) else Float.NaN,
                if (start.y.isFinite() && end.y.isFinite()) abs(start.y - end.y) else Float.NaN
            )

    override fun createShader(size: Size): Shader {
        val startX = if (start.x == Float.POSITIVE_INFINITY) size.width else start.x
        val startY = if (start.y == Float.POSITIVE_INFINITY) size.height else start.y
        val endX = if (end.x == Float.POSITIVE_INFINITY) size.width else end.x
        val endY = if (end.y == Float.POSITIVE_INFINITY) size.height else end.y
        return LinearGradientShader(
            colors = colors,
            colorStops = stops,
            from = Offset(startX, startY),
            to = Offset(endX, endY),
            tileMode = tileMode
        )
    }

    override fun toString(): String {
        val startValue = if (start.isFinite) "start=$start, " else ""
        val endValue = if (end.isFinite) "end=$end, " else ""
        return "LinearGradient(colors=$colors, " +
                "stops=$stops, " +
                startValue +
                endValue +
                "tileMode=$tileMode)"
    }
}

@Stable
fun texBrush(
    colors: List<Color>,
    startX: Float = 0.0f,
    endX: Float = Float.POSITIVE_INFINITY,
    tileMode: TileMode = TileMode.Clamp
): TexBrush {
    return TexBrush(colors, null, Offset(startX, 0.0f), Offset(endX, 0.0f), tileMode)
}
@Composable
fun texBrushUpdate(brush:TexBrush) {
    var sem = rt_regsiter()
    LaunchedEffect(brush) {
        withContext(Dispatchers.Default) {
            while(true) {
                sem.acquire()
                withContext(Dispatchers.Main) {
                    // update brush
                    println("brush update " + brush.toString())
                }
            }
        }
    }
    DisposableEffect(brush) {
        onDispose {
            // todo cleanup brush
            println("brush dispose " + brush.toString())
        }
    }
}

val global_id = atomic(0);
var notifier: MutableMap<Int, Semaphore> = mutableMapOf<Int, Semaphore>();
fun rt_regsiter():Semaphore {
    var sem = Semaphore(1)
    notifier[global_id.incrementAndGet()] = sem
    return sem
}
fun rt_update() {
    Thread {
        while(true) {
            Thread.sleep(1000)
            // Increment the counter
            notifier[1]?.release()
        }
    }.start()
}

@Composable
@Preview
fun App() {
    rt_update();
    MaterialTheme {
        var showContent by remember { mutableStateOf(false) }
        Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
            Button(onClick = { showContent = !showContent }) {
                Text("Click me!")
            }
            val brush by remember { mutableStateOf(texBrush(listOf(Color.Red, Color.Blue))) }
            texBrushUpdate(brush)
            Canvas(Modifier.fillMaxWidth().height(300.dp),
                //modifier = Modifier.size(200.dp),
                onDraw = {
                    drawRect(brush)
                })
            AnimatedVisibility(showContent) {
                val greeting = remember { Greeting().greet() }
                Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
                    Image(painterResource(Res.drawable.compose_multiplatform), null)
                    Text("Compose: $greeting")
                }
            }
        }
    }
}