package io.luoijin.ruo.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Button
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import io.luoijin.ruo.R
import kotlinx.serialization.Serializable

@Serializable
private data object HomeRoute

@Serializable
private data object SignInRoute

@Serializable
private data object ChatRoute

@Serializable
private data object HistoryRoute

@Serializable
private data object SettingsRoute

@Composable
fun RuoNavigation() {
    val navController = rememberNavController()
    NavHost(navController = navController, startDestination = HomeRoute) {
        composable<HomeRoute> {
            HomeScreen(
                onSignIn = { navController.navigate(SignInRoute) },
                onChat = { navController.navigate(ChatRoute) },
                onHistory = { navController.navigate(HistoryRoute) },
                onSettings = { navController.navigate(SettingsRoute) },
            )
        }
        composable<SignInRoute> {
            PlaceholderScreen(
                title = stringResource(R.string.sign_in_title),
                onBack = { navController.popBackStack() },
            )
        }
        composable<ChatRoute> {
            PlaceholderScreen(
                title = stringResource(R.string.chat_title),
                onBack = { navController.popBackStack() },
            )
        }
        composable<HistoryRoute> {
            PlaceholderScreen(
                title = stringResource(R.string.history_title),
                onBack = { navController.popBackStack() },
            )
        }
        composable<SettingsRoute> {
            PlaceholderScreen(
                title = stringResource(R.string.settings_title),
                onBack = { navController.popBackStack() },
            )
        }
    }
}

@Composable
private fun HomeScreen(
    onSignIn: () -> Unit,
    onChat: () -> Unit,
    onHistory: () -> Unit,
    onSettings: () -> Unit,
) {
    ShellSurface {
        Text(
            text = stringResource(R.string.app_name),
            style = MaterialTheme.typography.headlineMedium,
        )
        Text(
            text = stringResource(R.string.app_shell_description),
            style = MaterialTheme.typography.bodyLarge,
        )
        ShellButton(stringResource(R.string.sign_in_title), onSignIn)
        ShellButton(stringResource(R.string.chat_title), onChat)
        ShellButton(stringResource(R.string.history_title), onHistory)
        ShellButton(stringResource(R.string.settings_title), onSettings)
    }
}

@Composable
private fun PlaceholderScreen(title: String, onBack: () -> Unit) {
    ShellSurface {
        Text(text = title, style = MaterialTheme.typography.headlineMedium)
        Text(
            text = stringResource(R.string.placeholder_description),
            style = MaterialTheme.typography.bodyLarge,
        )
        ShellButton(stringResource(R.string.back), onBack)
    }
}

@Composable
private fun ShellSurface(content: @Composable () -> Unit) {
    Surface(
        modifier = Modifier.fillMaxSize(),
        color = MaterialTheme.colorScheme.background,
    ) {
        Column(
            modifier = Modifier
                .fillMaxSize()
                .padding(24.dp),
            verticalArrangement = Arrangement.spacedBy(16.dp),
        ) {
            content()
        }
    }
}

@Composable
private fun ShellButton(label: String, onClick: () -> Unit) {
    Button(
        modifier = Modifier.fillMaxWidth(),
        onClick = onClick,
    ) {
        Text(text = label)
    }
}
