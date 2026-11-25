package io.luoijin.ruo.ui.theme

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

private val RuoLightColors = lightColorScheme(
    primary = Color(0xFF1E4D4B),
    secondary = Color(0xFFD97706),
    background = Color(0xFFFAF9F6),
    surface = Color(0xFFFFFFFF),
    surfaceVariant = Color(0xFFF1EFEA),
    error = Color(0xFFB91C1C),
)

private val RuoDarkColors = darkColorScheme(
    primary = Color(0xFF5EEAD4),
    secondary = Color(0xFFFBBF24),
    background = Color(0xFF111315),
    surface = Color(0xFF1A1D21),
    surfaceVariant = Color(0xFF252930),
)

@Composable
fun RuoTheme(
    darkTheme: Boolean = isSystemInDarkTheme(),
    content: @Composable () -> Unit,
) {
    MaterialTheme(
        colorScheme = if (darkTheme) RuoDarkColors else RuoLightColors,
        content = content,
    )
}
