//! 日志服务模块
//!
//! 提供应用程序的日志记录功能，支持：
//! - 控制台输出（开发环境）
//! - 文件输出（生产环境）
//! - 日志轮转

use std::path::Path;
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};

/// 初始化日志系统
///
/// # Arguments
/// * `log_dir` - 日志文件存放目录
/// * `is_debug` - 是否为调试模式
///
/// # Returns
/// * `Ok(())` - 初始化成功
/// * `Err(String)` - 初始化失败的错误信息
pub fn init_logger(log_dir: &Path, is_debug: bool) -> Result<(), String> {
    // 确保日志目录存在
    std::fs::create_dir_all(log_dir)
        .map_err(|e| format!("创建日志目录失败: {}", e))?;

    // 创建文件日志输出（按天轮转）
    let file_appender = RollingFileAppender::new(Rotation::DAILY, log_dir, "gamemanager.log");

    // 设置日志级别过滤器
    let env_filter = if is_debug {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("debug,sqlx=warn,hyper=warn,reqwest=warn"))
    } else {
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info,sqlx=warn,hyper=warn,reqwest=warn"))
    };

    // 控制台输出层
    let console_layer = fmt::layer()
        .with_target(true)
        .with_thread_ids(false)
        .with_file(true)
        .with_line_number(true)
        .with_span_events(FmtSpan::CLOSE);

    // 文件输出层
    let file_layer = fmt::layer()
        .with_writer(file_appender)
        .with_ansi(false)
        .with_target(true)
        .with_file(true)
        .with_line_number(true);

    // 注册日志订阅器
    tracing_subscriber::registry()
        .with(env_filter)
        .with(console_layer)
        .with(file_layer)
        .try_init()
        .map_err(|e| format!("初始化日志系统失败: {}", e))?;

    tracing::info!("日志系统初始化完成");
    Ok(())
}

/// 记录游戏启动事件
#[inline]
pub fn log_game_launch(game_id: &str, game_title: &str, engine_type: &str) {
    tracing::info!(
        game_id = %game_id,
        title = %game_title,
        engine = %engine_type,
        "启动游戏"
    );
}

/// 记录游戏扫描事件
#[inline]
pub fn log_scan_start(root_path: &str, max_depth: u32) {
    tracing::info!(
        path = %root_path,
        max_depth = %max_depth,
        "开始扫描游戏目录"
    );
}

/// 记录游戏扫描结果
#[inline]
pub fn log_scan_complete(added: usize, skipped: usize, duration_ms: u64) {
    tracing::info!(
        added = %added,
        skipped = %skipped,
        duration_ms = %duration_ms,
        "游戏扫描完成"
    );
}

/// 记录图标提取事件
#[inline]
pub fn log_icon_extraction(game_id: &str, success: bool, error: Option<&str>) {
    if success {
        tracing::debug!(game_id = %game_id, "图标提取成功");
    } else {
        tracing::warn!(
            game_id = %game_id,
            error = error.unwrap_or("未知错误"),
            "图标提取失败"
        );
    }
}

/// 记录引擎检测结果
#[inline]
pub fn log_engine_detection(path: &str, engine: Option<&str>, confidence: i32) {
    match engine {
        Some(e) => {
            tracing::debug!(
                path = %path,
                engine = %e,
                confidence = %confidence,
                "检测到游戏引擎"
            );
        }
        None => {
            tracing::debug!(path = %path, "未能识别游戏引擎");
        }
    }
}

/// 记录错误
#[inline]
pub fn log_error(context: &str, error: &str) {
    tracing::error!(context = %context, error = %error, "操作失败");
}

/// 记录警告
#[inline]
pub fn log_warn(context: &str, message: &str) {
    tracing::warn!(context = %context, message = %message);
}

/// 记录调试信息
#[inline]
pub fn log_debug(context: &str, message: &str) {
    tracing::debug!(context = %context, message = %message);
}
