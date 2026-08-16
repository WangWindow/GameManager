use gamemanager_core::{AppPaths, GameManagerCore, WindowBackend};
use gamemanager_desktop::{DesktopApp, platform::DisplayBackendAvailability};

fn main() -> iced::Result {
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        run_with_window_backend(load_window_backend_preference())
    }

    #[cfg(not(all(unix, not(target_os = "macos"))))]
    {
        DesktopApp::run()
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn load_window_backend_preference() -> WindowBackend {
    let Ok(paths) = AppPaths::discover() else {
        return WindowBackend::Auto;
    };
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    else {
        return WindowBackend::Auto;
    };

    match runtime.block_on(GameManagerCore::read_ui_preferences(&paths)) {
        Ok(preferences) => preferences.window_backend,
        Err(error) => {
            eprintln!("GameManager: unable to read window backend preference: {error}");
            WindowBackend::Auto
        }
    }
}

#[cfg(all(unix, not(target_os = "macos")))]
fn run_with_window_backend(requested: WindowBackend) -> iced::Result {
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

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(DesktopApp::run));

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
