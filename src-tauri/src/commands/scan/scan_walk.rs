//! 扫描目录遍历：BFS 广度优先遍历目录树，统计有效子目录数量（跳过隐藏目录）。

use std::collections::VecDeque;
use std::path::{Path, PathBuf};

/// BFS 遍历统计根目录下（含自身）的文件系统目录数量，跳过 `.` 开头的隐藏目录。
pub(crate) fn count_dirs(root: &Path, max_depth: u32) -> u32 {
    let mut count = 0;
    let mut queue: VecDeque<(PathBuf, u32)> = VecDeque::new();
    queue.push_back((root.to_path_buf(), 0));

    while let Some((dir, depth)) = queue.pop_front() {
        count += 1;
        if depth >= max_depth {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&dir) {
            for entry in entries.flatten() {
                if let Ok(ty) = entry.file_type() {
                    if ty.is_dir() {
                        let name = entry.file_name().to_string_lossy().to_string();
                        if name.starts_with('.') {
                            continue;
                        }
                        queue.push_back((entry.path(), depth + 1));
                    }
                }
            }
        }
    }

    count.max(1)
}
