use std::{io, path::Path};

#[derive(Clone, Copy, Debug, Default)]
pub struct DesktopOpener;

impl DesktopOpener {
    pub fn open_path(self, path: &Path) -> io::Result<()> {
        if !path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                path.display().to_string(),
            ));
        }
        open::that_detached(path)
    }
}
