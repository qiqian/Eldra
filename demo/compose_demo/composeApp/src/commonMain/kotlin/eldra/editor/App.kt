package eldra.editor

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material.Button
import androidx.compose.material.MaterialTheme
import androidx.compose.material.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.geometry.isFinite
import androidx.compose.ui.graphics.*
import androidx.compose.ui.unit.Dp
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

val global_id = atomic(0);
var notifier: MutableMap<Int, Semaphore> = mutableMapOf<Int, Semaphore>();
fun ShaderRegister(sem:Semaphore) {
    notifier[global_id.incrementAndGet()] = sem
}
fun ShaderUpdate() {
    Thread {
        while(true) {
            Thread.sleep(1000)
            // Increment the counter
            notifier[1]?.release()
        }
    }.start()
}

@Composable
fun ShaderCanvas(width:Dp, height:Dp) {
    val brush by remember { mutableStateOf(texBrush(listOf(Color.Red, Color.Blue))) }
    val ver = remember { mutableStateOf(0) }
    val sem = Semaphore(1)
    ShaderRegister(sem)
    LaunchedEffect(brush) {
        withContext(Dispatchers.Default) {
            while(true) {
                sem.acquire()
                withContext(Dispatchers.Main) {
                    // update brush
                    ver.value++;
                    println("brush update " + ver.value.toString())
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
    var mod = if (width == 0.dp) { Modifier.fillMaxWidth() } else { Modifier.width(width) }
    mod = if (height == 0.dp) { mod.fillMaxHeight() } else { mod.height(height) }
    Canvas(mod.drawWithCache {
            val v = ver.value
            onDrawBehind {
                println("brush redraw " + v.toString())
                drawRect(brush)
            }
        },
        //modifier = Modifier.size(200.dp),
        onDraw = {
            //println("Shader brush!")
            //drawRect(brush)
        })
}

@Composable
@Preview
fun App() {
    ShaderUpdate();
    MaterialTheme {
        var showContent by remember { mutableStateOf(false) }
        Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
            Button(onClick = { showContent = !showContent }) {
                Text("Click me!")
            }
            ShaderCanvas(0.dp, 300.dp)
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