use std::path::Path;

pub(crate) const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "ico"];

pub(crate) fn is_image_file(path: &Path) -> bool {
    path.is_file()
        && path.extension().is_some_and(|extension| {
            IMAGE_EXTENSIONS
                .iter()
                .any(|accepted| extension.eq_ignore_ascii_case(accepted))
        })
}

pub(crate) fn sidecar_image(entry: &Path) -> Option<std::path::PathBuf> {
    let parent = entry.parent()?;
    let stem = entry.file_stem()?.to_str()?;
    let candidates = [
        stem.to_owned(),
        format!("{stem}-icon"),
        format!("{stem}_icon"),
        format!("{stem}Icon"),
    ];

    for candidate in candidates {
        for extension in IMAGE_EXTENSIONS {
            let path = parent.join(format!("{candidate}.{extension}"));
            if is_image_file(&path) {
                return Some(path);
            }
        }
    }
    None
}

pub(crate) fn image_in_icon_directories(game_root: &Path) -> Option<std::path::PathBuf> {
    image_in_directories(game_root, &["icon", "icons", "www/icon", "www/icons"])
}

pub(crate) fn conventional_cover(game_root: &Path) -> Option<std::path::PathBuf> {
    const CANDIDATES: &[&str] = &[
        "cover.png",
        "cover.jpg",
        "cover.jpeg",
        "cover.ico",
        "icon.png",
        "icon.jpg",
        "icon.jpeg",
        "icon.ico",
        "icon/cover.png",
        "icons/cover.png",
        "icon/cover.ico",
        "icons/cover.ico",
        "icon/icon.png",
        "icons/icon.png",
        "icon/icon.ico",
        "icons/icon.ico",
        "www/icon/cover.png",
        "www/icons/cover.png",
        "www/icon/cover.ico",
        "www/icons/cover.ico",
        "www/icon/icon.png",
        "www/icons/icon.png",
        "www/icon/icon.ico",
        "www/icons/icon.ico",
    ];

    CANDIDATES
        .iter()
        .map(|candidate| game_root.join(candidate))
        .find(|path| is_image_file(path))
        .or_else(|| image_in_icon_directories(game_root))
}

fn image_in_directories(game_root: &Path, directories: &[&str]) -> Option<std::path::PathBuf> {
    for directory in directories {
        let path = game_root.join(directory);
        let Ok(entries) = std::fs::read_dir(path) else {
            continue;
        };
        let mut images = entries
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| is_image_file(path))
            .collect::<Vec<_>>();
        images.sort();
        if let Some(image) = images.into_iter().next() {
            return Some(image);
        }
    }
    None
}
