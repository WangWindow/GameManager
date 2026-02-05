use std::ffi::OsStr;
use std::fs::File;
use std::path::{Component, Path, PathBuf};

/// 解压服务
/// 负责处理各种压缩格式的解压操作
pub struct ArchiveService;

impl ArchiveService {
    pub fn new() -> Self {
        Self
    }

    /// 解压 tar.gz 文件
    pub fn extract_tar_gz(&self, archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
        let file = File::open(archive_path)
            .map_err(|e| format!("无法打开文件 {}: {}", archive_path.display(), e))?;
        let gz = flate2::read::GzDecoder::new(file);
        let mut archive = tar::Archive::new(gz);

        for entry in archive
            .entries()
            .map_err(|e| format!("tar 读取错误: {}", e))?
        {
            let mut entry = entry.map_err(|e| format!("tar 条目错误: {}", e))?;
            let entry_path = entry
                .path()
                .map_err(|e| format!("tar 条目路径错误: {}", e))?
                .to_path_buf();

            let out_path = self.safe_join(dest_dir, &entry_path)?;
            if let Some(parent) = out_path.parent() {
                self.ensure_dir(parent)?;
            }
            entry
                .unpack(&out_path)
                .map_err(|e| format!("tar 解压错误: {}", e))?;
        }

        Ok(())
    }

    /// 解压 zip 文件
    pub fn extract_zip(&self, archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
        let file = File::open(archive_path)
            .map_err(|e| format!("无法打开文件 {}: {}", archive_path.display(), e))?;
        let mut zip = zip::ZipArchive::new(file).map_err(|e| format!("zip 打开错误: {}", e))?;

        for i in 0..zip.len() {
            let mut entry = zip
                .by_index(i)
                .map_err(|e| format!("zip 条目错误: {}", e))?;
            let name = entry
                .enclosed_name()
                .ok_or_else(|| "zip 条目中包含不安全的路径".to_string())?;
            let out_path = dest_dir.join(&name);

            if entry.is_dir() {
                self.ensure_dir(&out_path)?;
                continue;
            }

            if let Some(parent) = out_path.parent() {
                self.ensure_dir(parent)?;
            }

            let mut out = File::create(&out_path).map_err(|e| format!("zip 写入错误: {}", e))?;
            std::io::copy(&mut entry, &mut out).map_err(|e| format!("zip 解压错误: {}", e))?;
        }

        Ok(())
    }

    /// 根据文件扩展名自动选择解压方法
    pub fn extract_auto(&self, archive_path: &Path, dest_dir: &Path) -> Result<(), String> {
        let ext = archive_path
            .extension()
            .and_then(OsStr::to_str)
            .ok_or_else(|| "无法识别文件扩展名".to_string())?;

        // 检查是否是 .tar.gz
        let file_name = archive_path
            .file_name()
            .and_then(OsStr::to_str)
            .ok_or_else(|| "无效的文件名".to_string())?;

        if file_name.ends_with(".tar.gz") {
            return self.extract_tar_gz(archive_path, dest_dir);
        }

        match ext {
            "zip" => self.extract_zip(archive_path, dest_dir),
            "gz" => self.extract_tar_gz(archive_path, dest_dir),
            _ => Err(format!("不支持的压缩格式: {}", ext)),
        }
    }

    /// 查找解压后的单一根目录
    /// 许多压缩包会在顶层包含一个目录，此函数用于找到它
    pub fn find_single_root_dir(&self, dir: &Path) -> Option<PathBuf> {
        let mut entries = std::fs::read_dir(dir).ok()?;
        let mut only: Option<PathBuf> = None;

        while let Some(Ok(e)) = entries.next() {
            let p = e.path();
            // 忽略隐藏文件和系统文件
            if p.file_name()
                .and_then(OsStr::to_str)
                .unwrap_or("")
                .starts_with('.')
            {
                continue;
            }
            if only.is_some() {
                return None; // 找到多个条目
            }
            only = Some(p);
        }

        match only {
            Some(p) if p.is_dir() => Some(p),
            _ => None,
        }
    }

    /// 确保目录存在
    fn ensure_dir(&self, path: &Path) -> Result<(), String> {
        std::fs::create_dir_all(path).map_err(|e| format!("无法创建目录 {}: {}", path.display(), e))
    }

    /// 安全地连接路径，防止路径遍历攻击
    fn safe_join(&self, base: &Path, entry: &Path) -> Result<PathBuf, String> {
        let mut out = PathBuf::from(base);

        for comp in entry.components() {
            match comp {
                Component::Normal(p) => out.push(p),
                Component::CurDir => {}
                Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                    return Err(format!("压缩包中包含不安全的路径: {}", entry.display()));
                }
            }
        }

        Ok(out)
    }

    /// 如果目录存在则删除
    pub fn remove_dir_if_exists(&self, path: &Path) -> Result<(), String> {
        if path.exists() {
            std::fs::remove_dir_all(path)
                .map_err(|e| format!("无法删除目录 {}: {}", path.display(), e))?;
        }
        Ok(())
    }

    /// 移动目录
    pub fn move_dir(&self, src: &Path, dst: &Path) -> Result<(), String> {
        if let Some(parent) = dst.parent() {
            self.ensure_dir(parent)?;
        }

        // 优先使用 rename，跨设备时使用 copy + remove
        match std::fs::rename(src, dst) {
            Ok(()) => Ok(()),
            Err(_) => {
                self.copy_dir_all(src, dst)?;
                std::fs::remove_dir_all(src).map_err(|e| format!("无法清理临时目录: {}", e))?;
                Ok(())
            }
        }
    }

    /// 递归复制目录
    fn copy_dir_all(&self, src: &Path, dst: &Path) -> Result<(), String> {
        self.ensure_dir(dst)?;
        for entry in std::fs::read_dir(src).map_err(|e| format!("读取目录错误: {}", e))? {
            let entry = entry.map_err(|e| format!("读取目录条目错误: {}", e))?;
            let ty = entry
                .file_type()
                .map_err(|e| format!("获取文件类型错误: {}", e))?;
            let from = entry.path();
            let to = dst.join(entry.file_name());
            if ty.is_dir() {
                self.copy_dir_all(&from, &to)?;
            } else {
                std::fs::copy(&from, &to).map_err(|e| format!("复制文件错误: {}", e))?;
            }
        }
        Ok(())
    }
}

impl Default for ArchiveService {
    fn default() -> Self {
        Self::new()
    }
}
