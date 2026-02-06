// extension 模块，包含可选扩展服务（例如 Bottles）
// Bottles 的完整实现仅在 Linux 上编译，其它平台提供 stub

#[cfg(target_os = "linux")]
pub mod bottles;

pub mod integrations;

#[cfg(not(target_os = "linux"))]
pub mod bottles {
    use std::process;

    #[derive(Debug, Clone)]
    pub struct BottlesCli {
        pub program: String,
        pub args_prefix: Vec<String>,
    }

    impl BottlesCli {
        pub fn new(program: String, args_prefix: Vec<String>) -> Self {
            Self {
                program,
                args_prefix,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct BottlesService;

    impl BottlesService {
        pub async fn detect_cli() -> Option<BottlesCli> {
            None
        }

        pub fn detect_cli_sync() -> Option<BottlesCli> {
            None
        }

        pub async fn list_bottles(_cli: &BottlesCli) -> Result<Vec<String>, String> {
            Ok(Vec::new())
        }

        pub fn run_executable(
            _cli: &BottlesCli,
            _bottle: &str,
            _exe_path: &str,
            _args: &[String],
        ) -> Result<process::Child, String> {
            Err("Bottles 仅支持在 Linux 上运行".to_string())
        }
    }
}

// 导出 BottlesService 以便上层直接使用 `crate::services::BottlesService`
pub use bottles::BottlesService;
