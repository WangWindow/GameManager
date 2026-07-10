/// 路径工具 — 规范化、安全检查、目录操作。
use std::path::{Path, PathBuf};

/// 规范化路径，失败时返回原路径。
pub fn canonicalize(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// 确保目录存在，不存在则递归创建。
pub fn ensure_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| format!("创建目录失败 {}: {}", path.display(), e))
}

/// 判断 `path` 是否在 `root` 目录内（均先规范化）。
pub fn is_within(path: &Path, root: &Path) -> bool {
    canonicalize(path).starts_with(&canonicalize(root))
}

/// 判断文件是否为可直接启动的 Linux ELF 或带 shebang 的可执行脚本。
pub fn is_linux_native_executable(path: &Path) -> bool {
    if path
        .extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("exe"))
    {
        return false;
    }

    #[cfg(unix)]
    {
        use std::io::Read;
        use std::os::unix::fs::PermissionsExt;

        let Ok(metadata) = path.metadata() else {
            return false;
        };
        if !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
            return false;
        }

        let mut header = [0_u8; 4];
        let Ok(mut file) = std::fs::File::open(path) else {
            return false;
        };
        let Ok(read) = file.read(&mut header) else {
            return false;
        };
        return (read >= 4 && header == *b"\x7fELF") || (read >= 2 && header[..2] == *b"#!");
    }

    #[cfg(not(unix))]
    false
}
