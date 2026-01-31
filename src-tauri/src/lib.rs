//! YUML-DDT: YAML Data-Driven Testing Engine
//! 
//! Rust 重写的测试引擎，支持：
//! - YAML 配置文件解析
//! - 环境切换 (profile)
//! - 变量替换
//! - 路径映射
//! - HTTP 请求执行

pub mod engine;
pub mod config;
pub mod http_client;
pub mod cache;
pub mod commands;
pub mod storage;

pub use engine::TestEngine;
pub use config::*;
pub use commands::*;
pub use storage::*;
