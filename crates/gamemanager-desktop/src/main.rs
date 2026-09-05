use gamemanager_core::{AppPaths, GameManagerCore, UiPreferences, WindowBackend};
use gamemanager_desktop::{DesktopApp, platform::DisplayBackendAvailability};

fn main() -> iced::Result {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let preferences = load_ui_preferences();
        let backend = preferences
            .as_ref()
            .map_or(WindowBackend::Auto, |preferences| {
                preferences.window_backend
            });
        let window_size = initial_window_size(preferences.as_ref());
        run_with_window_backend(backend, window_size)
    }

    #[cfg(not(all(unix, not(target_os = "macos"))))]
    {
        DesktopApp::run()
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn load_ui_preferences() -> Option<UiPreferences> {
    let Ok(paths) = AppPaths::discover() else {
        return None;
    };
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return None;
    };

    match runtime.block_on(GameManagerCore::read_ui_preferences(&paths)) {
        Ok(preferences) => Some(preferences),
        Err(error) => {
            eprintln!("GameManager: unable to read UI preferences: {error}");
            None
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn initial_window_size(preferences: Option<&UiPreferences>) -> iced::Size {
    let fallback = iced::window::Settings::default().size;
    preferences
        .filter(|preferences| preferences.remember_window_size)
        .and_then(|preferences| preferences.window_size)
        .map_or(fallback, |[width, height]| {
            iced::Size::new(width as f32, height as f32)
        })
}

#[cfg(all(unix, not(target_os = "macos")))]
fn run_with_window_backend(
    requested: WindowBackend,
    initial_window_size: iced::Size,
) -> iced::Result {
    let availability = DisplayBackendAvailability::detect();
    let backend = match availability.resolve(requested) {
        WindowBackend::Auto if DisplayBackendAvailability::wayland_endpoint_present() => {
            WindowBackend::Wayland
        }
        WindowBackend::Auto if DisplayBackendAvailability::x11_endpoint_present() => {
            WindowBackend::X11
        }
        backend => backend,
    };

    // Environment variables are process-global and changing them in a Rust
    // 2024 process is unsafe. Re-exec once when an explicit X11 choice must
    // override a Wayland session instead.
    if backend == WindowBackend::X11
        && DisplayBackendAvailability::wayland_endpoint_present()
        && DisplayBackendAvailability::x11_endpoint_present()
    {
        match relaunch_with_x11() {
            Ok(status) => std::process::exit(status.code().unwrap_or(1)),
            Err(error) => {
                eprintln!("GameManager: unable to switch to X11: {error}");
            }
        }
    }

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        DesktopApp::run_with_initial_window_size(initial_window_size)
    }));

    match result {
        Ok(result) => {
            if backend == WindowBackend::Wayland
                && result.is_err()
                && DisplayBackendAvailability::x11_endpoint_present()
            {
                relaunch_with_x11_or_return(result)
            } else {
                result
            }
        }
        Err(payload) => {
            if backend == WindowBackend::Wayland
                && startup_panic(&payload)
                && DisplayBackendAvailability::x11_endpoint_present()
            {
                match relaunch_with_x11() {
                    Ok(status) => std::process::exit(status.code().unwrap_or(1)),
                    Err(error) => {
                        eprintln!(
                            "GameManager: Wayland startup failed and X11 fallback could not start: {error}"
                        );
                    }
                }
            }

            std::panic::resume_unwind(payload);
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn startup_panic(payload: &Box<dyn std::any::Any + Send>) -> bool {
    let message = payload
        .downcast_ref::<&str>()
        .copied()
        .or_else(|| payload.downcast_ref::<String>().map(String::as_str));

    message.is_some_and(|message| {
        message.contains("Create event loop") || message.contains("Create window")
    })
}

#[cfg(all(unix, not(target_os = "macos")))]
fn relaunch_with_x11_or_return(result: iced::Result) -> iced::Result {
    match relaunch_with_x11() {
        Ok(status) => std::process::exit(status.code().unwrap_or(1)),
        Err(error) => {
            eprintln!(
                "GameManager: Wayland startup failed and X11 fallback could not start: {error}"
            );
            result
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn relaunch_with_x11() -> std::io::Result<std::process::ExitStatus> {
    let executable = std::env::current_exe()?;
    std::process::Command::new(executable)
        .args(std::env::args_os().skip(1))
        // Preserve the capability detected by the parent process for the
        // settings view. This marker does not affect Winit's backend choice.
        .env("GAMEMANAGER_WAYLAND_AVAILABLE", "1")
        // Winit selects Wayland whenever these variables are present. Remove
        // them in the child and let DISPLAY select X11/XWayland instead.
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("WAYLAND_SOCKET")
        .status()
}

#[cfg(all(unix, not(target_os = "macos")))]
#[cfg(test)]
mod tests {
    use super::startup_panic;

    #[test]
    fn only_backend_initialization_panics_trigger_fallback() {
        let event_loop =
            Box::new("Create event loop: unavailable") as Box<dyn std::any::Any + Send>;
        let unrelated = Box::new("application task failed") as Box<dyn std::any::Any + Send>;

        assert!(startup_panic(&event_loop));
        assert!(!startup_panic(&unrelated));
    }
}
