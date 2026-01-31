//! 配置文件数据结构定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 完整的测试配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub global: GlobalConfig,
    #[serde(default)]
    pub variables: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub steps: HashMap<String, Step>,
    #[serde(default)]
    pub test_cases: HashMap<String, TestCase>,
}

/// 全局配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default)]
    pub debug: bool,
    #[serde(default)]
    pub profile: ProfileConfig,
}

/// Profile 配置
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProfileConfig {
    #[serde(default)]
    pub active: String,
    #[serde(flatten)]
    pub environments: HashMap<String, EnvironmentConfig>,
}

/// 环境配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub context: String,
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub path_mapping: HashMap<String, String>,
    #[serde(default)]
    pub auth: Option<AuthConfig>,
}

/// 认证配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default)]
    pub token_url: String,
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub grant_type: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub token_cache_key: String,
    #[serde(default)]
    pub auth_position: String,  // header/query/body
    #[serde(default)]
    pub auth_key: String,
    #[serde(default)]
    pub auth_prefix: String,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token_url: String::new(),
            client_id: String::new(),
            grant_type: String::new(),
            username: String::new(),
            password: String::new(),
            token_cache_key: String::new(),
            auth_position: "header".to_string(),
            auth_key: "Authorization".to_string(),
            auth_prefix: "Bearer".to_string(),
        }
    }
}

/// 测试步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub params: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub body: Option<serde_yaml::Value>,
    #[serde(default)]
    pub headers: HashMap<String, String>,
    #[serde(default)]
    pub variables: HashMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub validate: Vec<Validation>,
    #[serde(default)]
    pub save_response: Vec<SaveRule>,
}

/// 验证规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validation {
    pub field: String,
    pub operator: String,
    pub expected: serde_yaml::Value,
}

/// 保存响应规则
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveRule {
    pub field: String,
    pub to: String,
}

/// 测试用例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub steps: Vec<String>,
    #[serde(default)]
    pub variables: HashMap<String, serde_yaml::Value>,
}

/// Step 信息（用于前端展示）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepInfo {
    pub id: String,
    pub name: String,
    pub method: String,
    pub path: String,
}

/// 执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub step_name: String,
    pub request_url: String,
    pub request_method: String,
    pub request_headers: HashMap<String, String>,
    pub request_body: Option<serde_json::Value>,
    pub response_status: u16,
    pub response_headers: HashMap<String, String>,
    pub response_body: Option<serde_json::Value>,
    pub validations: Vec<ValidationResult>,
    pub duration_ms: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub field: String,
    pub operator: String,
    pub expected: String,
    pub actual: String,
    pub passed: bool,
}
