use std::path::PathBuf;

#[derive(Clone, Copy, Debug, Default)]
pub struct DesktopDialog;

impl DesktopDialog {
    pub async fn pick_file(self) -> Option<PathBuf> {
        rfd::AsyncFileDialog::new()
            .pick_file()
            .await
            .map(|file| file.path().to_path_buf())
    }

    pub async fn pick_directory(self) -> Option<PathBuf> {
        rfd::AsyncFileDialog::new()
            .pick_folder()
            .await
            .map(|file| file.path().to_path_buf())
    }

    pub async fn pick_image_file(self) -> Option<PathBuf> {
        rfd::AsyncFileDialog::new()
            .add_filter("Image", &["png", "jpg", "jpeg", "webp", "ico"])
            .pick_file()
            .await
            .map(|file| file.path().to_path_buf())
    }
}
