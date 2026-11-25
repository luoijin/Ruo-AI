package io.luoijin.ruo

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import dagger.hilt.android.AndroidEntryPoint
import io.luoijin.ruo.ui.RuoNavigation
import io.luoijin.ruo.ui.theme.RuoTheme

@AndroidEntryPoint
class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContent {
            RuoTheme {
                RuoNavigation()
            }
        }
    }
}
