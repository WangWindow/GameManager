/// 路径工具 — 规范化、安全检查、目录操作。
use std::path::{Path, PathBuf};

/// 规范化路径，失败时返回原路径。
pub fn canonicalize(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// 确保目录存在，不存在则递归创建。
pub fn ensure_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path)
        .map_err(|e| format!("创建目录失败 {}: {}", path.display(), e))
}

/// 判断 `path` 是否在 `root` 目录内（均先规范化）。
pub fn is_within(path: &Path, root: &Path) -> bool {
    canonicalize(path).starts_with(&canonicalize(root))
}
