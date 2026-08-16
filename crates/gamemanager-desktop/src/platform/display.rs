use gamemanager_core::WindowBackend;

/// Set only on the X11 re-exec child so the settings view can keep showing
/// the Wayland capability that was detected before `WAYLAND_DISPLAY` had to
/// be removed for Winit's backend selection.
#[cfg(all(unix, not(target_os = "macos")))]
const WAYLAND_AVAILABLE_MARKER: &str = "GAMEMANAGER_WAYLAND_AVAILABLE";

/// Display endpoints exposed by the current desktop session.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DisplayBackendAvailability {
    pub wayland: bool,
    pub x11: bool,
}

impl DisplayBackendAvailability {
    pub fn wayland_endpoint_present() -> bool {
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            std::env::var_os("WAYLAND_DISPLAY").is_some()
        }

        #[cfg(not(all(unix, not(target_os = "macos"))))]
        {
            false
        }
    }

    pub fn x11_endpoint_present() -> bool {
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            std::env::var_os("DISPLAY").is_some()
        }

        #[cfg(not(all(unix, not(target_os = "macos"))))]
        {
            false
        }
    }

    pub fn detect() -> Self {
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            Self {
                wayland: Self::wayland_endpoint_present()
                    || std::env::var_os(WAYLAND_AVAILABLE_MARKER).is_some(),
                x11: Self::x11_endpoint_present(),
            }
        }

        #[cfg(not(all(unix, not(target_os = "macos"))))]
        {
            Self::default()
        }
    }

    pub const fn default_backend(self) -> WindowBackend {
        if self.wayland {
            WindowBackend::Wayland
        } else if self.x11 {
            WindowBackend::X11
        } else {
            WindowBackend::Auto
        }
    }

    pub const fn resolve(self, requested: WindowBackend) -> WindowBackend {
        match requested {
            WindowBackend::Wayland if self.wayland => WindowBackend::Wayland,
            WindowBackend::X11 if self.x11 => WindowBackend::X11,
            WindowBackend::Auto | WindowBackend::Wayland | WindowBackend::X11 => {
                self.default_backend()
            }
        }
    }

    pub const fn supports(self, backend: WindowBackend) -> bool {
        match backend {
            WindowBackend::Auto => true,
            WindowBackend::Wayland => self.wayland,
            WindowBackend::X11 => self.x11,
        }
    }
}

#[cfg(test)]
mod tests {
    use gamemanager_core::WindowBackend;

    use super::DisplayBackendAvailability;

    #[test]
    fn wayland_is_the_default_when_both_endpoints_exist() {
        let availability = DisplayBackendAvailability {
            wayland: true,
            x11: true,
        };

        assert_eq!(availability.default_backend(), WindowBackend::Wayland);
        assert_eq!(availability.resolve(WindowBackend::X11), WindowBackend::X11);
    }

    #[test]
    fn unavailable_preferences_use_the_available_backend() {
        let availability = DisplayBackendAvailability {
            wayland: false,
            x11: true,
        };

        assert!(!availability.supports(WindowBackend::Wayland));
        assert_eq!(
            availability.resolve(WindowBackend::Wayland),
            WindowBackend::X11
        );
    }
}
