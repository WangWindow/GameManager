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
    ///
    /// 支持:
    /// - Windows PE 文件 (.exe)
    /// - 游戏目录中的图片文件作为 fallback
    ///
    /// 图标会被转换为 PNG 格式以获得更好的兼容性
    pub fn save_exe_icon_to_profile(
        &self,
        container_root: &Path,
        profile_key: &str,
        exe_path: &Path,
    ) -> Option<PathBuf> {
        let game_dir = exe_path.parent()?;

        // 策略1: 尝试从可执行文件提取图标
        if let Some(icon_data) = self.extract_pe_icon(exe_path) {
            if let Some(path) = self.save_icon_data_to_profile(
                container_root,
                profile_key,
                &icon_data,
                "exe",
            ) {
                tracing::debug!(profile_key = %profile_key, "从可执行文件提取图标成功");
                return Some(path);
            }
        }

        // 策略2: 尝试查找同名图片文件
        if let Some(sidecar) = self.find_sidecar_icon(exe_path) {
            if let Ok(path) = self.save_cover_to_profile(container_root, profile_key, &sidecar) {
                tracing::debug!(profile_key = %profile_key, "使用同名图片文件");
                return Some(path);
            }
        }

        // 策略3: 在 icon/icons 目录中查找
        if let Some(icon_image) = self.find_icon_dir_image(game_dir) {
            if let Ok(path) = self.save_cover_to_profile(container_root, profile_key, &icon_image) {
                tracing::debug!(profile_key = %profile_key, "使用 icon 目录图片");
                return Some(path);
            }
        }

        // 策略4: 使用游戏目录中的封面图
        if let Some(cover) = self.find_cover_image(game_dir) {
            if let Ok(path) = self.save_cover_to_profile(container_root, profile_key, &cover) {
                tracing::debug!(profile_key = %profile_key, "使用游戏目录封面图");
                return Some(path);
            }
        }

        tracing::debug!(profile_key = %profile_key, "未能找到或提取图标");
        None
    }

    /// 查找与可执行文件同名的图片
    fn find_sidecar_icon(&self, exe_path: &Path) -> Option<PathBuf> {
        let parent = exe_path.parent()?;
        let stem = exe_path.file_stem()?.to_str()?;

        let extensions = ["png", "ico", "jpg", "jpeg", "webp"];
        let patterns = [
            format!("{}", stem),
            format!("{}-icon", stem),
            format!("{}_icon", stem),
            format!("{}Icon", stem),
        ];

        for pattern in &patterns {
            for ext in &extensions {
                let candidate = parent.join(format!("{}.{}", pattern, ext));
                if candidate.exists() && candidate.is_file() {
                    return Some(candidate);
                }
            }
        }

        None
    }

    /// 保存图标数据到 profile 目录
    fn save_icon_data_to_profile(
        &self,
        container_root: &Path,
        profile_key: &str,
        icon_data: &[u8],
        source: &str,
    ) -> Option<PathBuf> {
        let profile_dir = self.game_profile_dir(container_root, profile_key);
        ensure_dir(&profile_dir).ok()?;

        // 尝试将 ICO 转换为 PNG 以获得更好的兼容性
        let (data, ext) = self.convert_icon_to_png(icon_data).unwrap_or_else(|| {
            (icon_data.to_vec(), "ico")
        });

        let target = profile_dir.join(format!("cover.{}", ext));
        std::fs::write(&target, data).ok()?;

        tracing::debug!(
            profile_key = %profile_key,
            source = %source,
            format = %ext,
            "保存图标成功"
        );

        Some(target)
    }

    /// 将 ICO 格式转换为 PNG
    fn convert_icon_to_png(&self, ico_data: &[u8]) -> Option<(Vec<u8>, &'static str)> {
        use std::io::Cursor;

        // 尝试加载 ICO 文件
        let img = image::load_from_memory_with_format(ico_data, image::ImageFormat::Ico).ok()?;

        // 编码为 PNG
        let mut png_data = Vec::new();
        let mut cursor = Cursor::new(&mut png_data);
        img.write_to(&mut cursor, image::ImageFormat::Png).ok()?;

        Some((png_data, "png"))
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

    /// 从 PE 文件提取图标
    ///
    /// 支持 PE32 和 PE64 格式的 Windows 可执行文件
    fn extract_pe_icon(&self, exe_path: &Path) -> Option<Vec<u8>> {
        // 检查文件扩展名
        let ext = exe_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        if ext != "exe" {
            tracing::debug!(path = %exe_path.display(), "非 exe 文件，跳过图标提取");
            return None;
        }

        let file = match pelite::FileMap::open(exe_path) {
            Ok(f) => f,
            Err(e) => {
                tracing::debug!(
                    path = %exe_path.display(),
                    error = %e,
                    "无法打开 PE 文件"
                );
                return None;
            }
        };

        let bytes = file.as_ref();

        // 尝试解析为 PE64
        if let Ok(pe) = pelite::pe64::PeFile::from_bytes(bytes) {
            if let Ok(resources) = pe.resources() {
                if let Some(icon) = self.extract_pe_icon_from_resources(resources) {
                    return Some(icon);
                }
            }
        }

        // 尝试解析为 PE32
        if let Ok(pe) = pelite::pe32::PeFile::from_bytes(bytes) {
            if let Ok(resources) = pe.resources() {
                if let Some(icon) = self.extract_pe_icon_from_resources(resources) {
                    return Some(icon);
                }
            }
        }

        tracing::debug!(path = %exe_path.display(), "PE 文件中未找到图标资源");
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
