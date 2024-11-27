package eldra.editor

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
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
import androidx.compose.ui.graphics.Brush.Companion.linearGradient
import androidx.compose.ui.graphics.LinearGradient
import androidx.compose.ui.unit.dp
import org.jetbrains.compose.resources.painterResource
import org.jetbrains.compose.ui.tooling.preview.Preview

import kotlin.math.abs

import compose_demo.composeapp.generated.resources.Res
import compose_demo.composeapp.generated.resources.compose_multiplatform

@Immutable
class LinearGradient2 internal constructor(
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

    override fun hashCode(): Int {
        var result = colors.hashCode()
        result = 31 * result + (stops?.hashCode() ?: 0)
        result = 31 * result + start.hashCode()
        result = 31 * result + end.hashCode()
        result = 31 * result + tileMode.hashCode()
        return result
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
fun horizontalGradient2(
    colors: List<Color>,
    startX: Float = 0.0f,
    endX: Float = Float.POSITIVE_INFINITY,
    tileMode: TileMode = TileMode.Clamp
): Brush = linearGradient2(colors, Offset(startX, 0.0f), Offset(endX, 0.0f), tileMode)

@Stable
fun linearGradient2(
    colors: List<Color>,
    start: Offset = Offset.Zero,
    end: Offset = Offset.Infinite,
    tileMode: TileMode = TileMode.Clamp
): Brush = LinearGradient2(
    colors = colors,
    stops = null,
    start = start,
    end = end,
    tileMode = tileMode
)

@Composable
@Preview
fun App() {
    MaterialTheme {
        var showContent by remember { mutableStateOf(false) }
        Column(Modifier.fillMaxWidth(), horizontalAlignment = Alignment.CenterHorizontally) {
            Button(onClick = { showContent = !showContent }) {
                Text("Click me!")
            }
            val brush = horizontalGradient2(listOf(Color.Red, Color.Blue))
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