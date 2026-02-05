use pelite::pe32::Pe as Pe32;
use pelite::pe64::Pe as Pe64;
use std::path::{Path, PathBuf};

/// 确保目录存在，不存在则创建
pub fn ensure_dir(path: &Path) -> Result<(), String> {
    std::fs::create_dir_all(path).map_err(|e| format!("创建目录失败 {}: {}", path.display(), e))
}

/// 规范化路径（失败时返回原路径）
pub fn canonicalize_path(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// 判断路径是否在根目录内
pub fn is_within_dir(path: &Path, root: &Path) -> bool {
    let path = canonicalize_path(path);
    let root = canonicalize_path(root);
    path.starts_with(root)
}

/// 文件管理服务
pub struct FileService;

impl FileService {
    /// 创建文件服务实例
    pub fn new() -> Self {
        Self
    }

    /// 获取游戏配置目录
    pub fn game_profile_dir(&self, container_root: &Path, profile_key: &str) -> PathBuf {
        container_root.join("profiles").join(profile_key)
    }

    /// 获取游戏用户数据目录
    pub fn game_user_data_dir(&self, container_root: &Path, profile_key: &str) -> PathBuf {
        self.game_profile_dir(container_root, profile_key)
            .join("User Data")
    }

    /// 获取崩溃报告目录
    pub fn game_crash_dir(&self, container_root: &Path, profile_key: &str) -> PathBuf {
        self.game_profile_dir(container_root, profile_key)
            .join("Crash Reports")
    }

    /// 获取游戏配置文件路径
    pub fn game_config_path(&self, container_root: &Path, profile_key: &str) -> PathBuf {
        self.game_profile_dir(container_root, profile_key)
            .join("settings.toml")
    }

    /// 确保游戏目录结构存在
    pub fn ensure_game_dirs(&self, container_root: &Path, profile_key: &str) -> Result<(), String> {
        let profile_dir = self.game_profile_dir(container_root, profile_key);
        let user_data_dir = self.game_user_data_dir(container_root, profile_key);
        let crash_dir = self.game_crash_dir(container_root, profile_key);

        ensure_dir(&profile_dir)?;
        ensure_dir(&user_data_dir)?;
        ensure_dir(&crash_dir)?;

        Ok(())
    }

    /// 迁移profile目录名称
    pub fn migrate_profile_dir(
        &self,
        container_root: &Path,
        old_key: &str,
        new_key: &str,
    ) -> Result<(), String> {
        if old_key == new_key {
            return Ok(());
        }
        let old_dir = self.game_profile_dir(container_root, old_key);
        let new_dir = self.game_profile_dir(container_root, new_key);

        if old_dir.exists() && !new_dir.exists() {
            std::fs::rename(&old_dir, &new_dir)
                .map_err(|e| format!("迁移profile目录失败: {}", e))?;
        }

        Ok(())
    }

    /// 查找游戏封面图片
    pub fn find_cover_image(&self, game_path: &Path) -> Option<PathBuf> {
        // 尝试多个可能的封面位置
        let candidates = [
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

        for candidate in &candidates {
            let path = game_path.join(candidate);
            if path.exists() && path.is_file() {
                return Some(path);
            }
        }

        // 尝试在icon目录中查找任何图片
        self.find_image_in_dirs(game_path, &["icon", "icons", "www/icon", "www/icons"])
    }

    /// 在icon目录中查找图片
    pub fn find_icon_dir_image(&self, game_path: &Path) -> Option<PathBuf> {
        self.find_image_in_dirs(game_path, &["icon", "icons", "www/icon", "www/icons"])
    }

    /// 从可执行文件提取图标并保存到profile目录
    pub fn save_exe_icon_to_profile(
        &self,
        container_root: &Path,
        profile_key: &str,
        exe_path: &Path,
    ) -> Option<PathBuf> {
        let temp_dir = tempfile::tempdir().ok()?;
        let extracted = self.extract_exe_icon_to_dir(exe_path, temp_dir.path())?;
        self.save_cover_to_profile(container_root, profile_key, &extracted)
            .ok()
    }

    /// 保存封面到profile目录并返回保存路径
    pub fn save_cover_to_profile(
        &self,
        container_root: &Path,
        profile_key: &str,
        source_path: &Path,
    ) -> Result<PathBuf, String> {
        let profile_dir = self.game_profile_dir(container_root, profile_key);
        ensure_dir(&profile_dir)?;

        let ext = source_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let target = profile_dir.join(format!("cover.{ext}"));

        std::fs::copy(source_path, &target).map_err(|e| format!("保存封面失败: {}", e))?;

        Ok(target)
    }

    /// 在指定目录中查找图片文件
    fn find_image_in_dirs(&self, base_path: &Path, dirs: &[&str]) -> Option<PathBuf> {
        for dir in dirs {
            let dir_path = base_path.join(dir);
            if !dir_path.is_dir() {
                continue;
            }

            if let Ok(entries) = std::fs::read_dir(&dir_path) {
                for entry in entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_file() {
                            let path = entry.path();
                            if self.is_image_file(&path) {
                                return Some(path);
                            }
                        }
                    }
                }
            }
        }

        None
    }

    /// 检查文件是否为图片
    fn is_image_file(&self, path: &Path) -> bool {
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext.to_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "webp" | "bmp" | "ico"
            )
        } else {
            false
        }
    }

    fn extract_exe_icon_to_dir(&self, exe_path: &Path, out_dir: &Path) -> Option<PathBuf> {
        let icon = self.extract_pe_icon(exe_path)?;
        let target = out_dir.join("icon.ico");
        std::fs::write(&target, icon).ok()?;
        Some(target)
    }

    fn extract_pe_icon(&self, exe_path: &Path) -> Option<Vec<u8>> {
        let file = pelite::FileMap::open(exe_path).ok()?;
        let bytes = file.as_ref();

        if let Ok(pe) = pelite::pe64::PeFile::from_bytes(bytes) {
            return self.extract_pe_icon_from_resources(pe.resources().ok()?);
        }

        if let Ok(pe) = pelite::pe32::PeFile::from_bytes(bytes) {
            return self.extract_pe_icon_from_resources(pe.resources().ok()?);
        }

        None
    }

    fn extract_pe_icon_from_resources(
        &self,
        res: pelite::resources::Resources<'_>,
    ) -> Option<Vec<u8>> {
        let mut icons = res.icons().filter_map(Result::ok);
        let (_name, group) = icons.next()?;
        let mut out = Vec::new();
        group.write(&mut out).ok()?;
        Some(out)
    }

    /// 读取游戏配置
    pub fn read_game_config(
        &self,
        config_path: &Path,
    ) -> Result<crate::models::GameConfig, String> {
        let content =
            std::fs::read_to_string(config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

        let config: crate::models::GameConfig =
            toml::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

        Ok(config)
    }

    /// 保存游戏配置
    pub fn write_game_config(
        &self,
        config_path: &Path,
        config: &crate::models::GameConfig,
    ) -> Result<(), String> {
        let content =
            toml::to_string_pretty(config).map_err(|e| format!("序列化配置失败: {}", e))?;

        std::fs::write(config_path, content).map_err(|e| format!("写入配置文件失败: {}", e))?;

        Ok(())
    }
}

impl Default for FileService {
    fn default() -> Self {
        Self::new()
    }
}
