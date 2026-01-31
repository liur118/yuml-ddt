//! 测试引擎核心实现

use regex::Regex;
use serde_json::Value as JsonValue;
use serde_yaml::Value as YamlValue;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

use crate::config::*;
use crate::http_client::HttpClient;

pub struct TestEngine {
    config: TestConfig,
    env_config: EnvironmentConfig,
    client: HttpClient,
    context: HashMap<String, JsonValue>,
    debug: bool,
    path_mapping: HashMap<String, String>,
}

impl TestEngine {
    /// 从配置文件创建引擎
    pub fn from_file(config_path: &str) -> Result<Self, String> {
        let content = fs::read_to_string(config_path)
            .map_err(|e| format!("无法读取配置文件: {}", e))?;
        
        Self::from_yaml(&content)
    }

    /// 从 YAML 字符串创建引擎
    pub fn from_yaml(yaml_content: &str) -> Result<Self, String> {
        let config: TestConfig = serde_yaml::from_str(yaml_content)
            .map_err(|e| format!("YAML 解析错误: {}", e))?;
        
        Self::new(config)
    }

    /// 从配置创建引擎
    pub fn new(config: TestConfig) -> Result<Self, String> {
        let debug = config.global.debug;
        
        // 解析环境配置
        let env_config = Self::resolve_env_config(&config)?;
        let path_mapping = env_config.path_mapping.clone();
        
        // 创建 HTTP 客户端
        let client = HttpClient::new(
            env_config.base_url.clone(),
            env_config.auth.clone(),
            debug,
        );
        
        // 初始化上下文
        let mut context = HashMap::new();
        context.insert("brand".to_string(), JsonValue::String(env_config.brand.clone()));
        context.insert("timestamp".to_string(), JsonValue::String(
            chrono::Utc::now().timestamp().to_string()
        ));
        
        Ok(Self {
            config,
            env_config,
            client,
            context,
            debug,
            path_mapping,
        })
    }

    /// 解析环境配置
    fn resolve_env_config(config: &TestConfig) -> Result<EnvironmentConfig, String> {
        let active = &config.global.profile.active;
        
        if active.is_empty() {
            return Err("未配置 profile.active".to_string());
        }
        
        config.global.profile.environments
            .get(active)
            .cloned()
            .ok_or_else(|| format!("环境 '{}' 配置不存在", active))
    }

    /// 获取 step 列表
    pub fn get_step_list(&self) -> Vec<StepInfo> {
        self.config.steps
            .iter()
            .map(|(id, step)| StepInfo {
                id: id.clone(),
                name: if step.name.is_empty() { id.clone() } else { step.name.clone() },
                method: step.method.clone(),
                path: step.path.clone(),
            })
            .collect()
    }

    /// 执行单个步骤
    pub async fn execute_step(
        &mut self,
        step_name: &str,
        external_variables: Option<HashMap<String, JsonValue>>,
    ) -> ExecutionResult {
        let start = Instant::now();
        
        // 获取 step 配置
        let step = match self.config.steps.get(step_name) {
            Some(s) => s.clone(),
            None => {
                return ExecutionResult {
                    success: false,
                    step_name: step_name.to_string(),
                    request_url: String::new(),
                    request_method: String::new(),
                    request_headers: HashMap::new(),
                    request_body: None,
                    response_status: 0,
                    response_headers: HashMap::new(),
                    response_body: None,
                    validations: vec![],
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: Some(format!("步骤 '{}' 不存在", step_name)),
                };
            }
        };

        // 合并变量：全局 < step级别 < 外部传入
        let mut merged_vars = self.context.clone();
        
        // 添加全局变量
        for (k, v) in &self.config.variables {
            merged_vars.insert(k.clone(), yaml_to_json(v));
        }
        
        // 添加 step 级别变量
        for (k, v) in &step.variables {
            merged_vars.insert(k.clone(), yaml_to_json(v));
        }
        
        // 添加外部变量
        if let Some(ext_vars) = external_variables {
            merged_vars.extend(ext_vars);
        }

        // 处理路径
        let raw_path = &step.path;
        let full_path = format!("{}{}", self.env_config.context, raw_path);
        let full_path = self.replace_variables(&full_path, &merged_vars);
        let path = self.transform_path(&full_path);

        // 处理请求参数
        let params = self.replace_map_variables(&step.params, &merged_vars);
        let params_str: HashMap<String, String> = params
            .iter()
            .map(|(k, v)| (k.clone(), json_value_to_string(v)))
            .collect();

        // 处理请求体
        let body = step.body.as_ref().map(|b| {
            let json_body = yaml_to_json(b);
            self.replace_json_variables(&json_body, &merged_vars)
        });

        // 处理请求头
        let headers: HashMap<String, String> = step.headers
            .iter()
            .map(|(k, v)| (k.clone(), self.replace_variables(v, &merged_vars)))
            .collect();

        let request_url = format!("{}{}", self.env_config.base_url, path);
        let method = step.method.to_uppercase();

        if self.debug {
            println!("📤 请求: {} {}", method, request_url);
            if !params_str.is_empty() {
                println!("   参数: {:?}", params_str);
            }
            if let Some(ref b) = body {
                println!("   请求体: {}", serde_json::to_string_pretty(b).unwrap_or_default());
            }
        }

        // 发送请求
        let result = match method.as_str() {
            "GET" => self.client.get(&path, Some(&params_str), Some(&headers)).await,
            "POST" => self.client.post(&path, body.as_ref(), Some(&params_str), Some(&headers)).await,
            "PUT" => self.client.put(&path, body.as_ref(), Some(&params_str), Some(&headers)).await,
            "DELETE" => self.client.delete(&path, Some(&params_str), Some(&headers)).await,
            _ => Err(format!("不支持的 HTTP 方法: {}", method)),
        };

        match result {
            Ok((status, actual_request_headers, response_headers, response_body)) => {
                // 验证响应
                let validations = self.validate_response(&response_body, &step.validate);
                let all_passed = validations.iter().all(|v| v.passed);

                // 保存响应数据
                for rule in &step.save_response {
                    if let Some(value) = get_json_field(&response_body, &rule.field) {
                        self.context.insert(rule.to.clone(), value);
                    }
                }

                ExecutionResult {
                    success: all_passed && status.is_success(),
                    step_name: step_name.to_string(),
                    request_url,
                    request_method: method,
                    request_headers: actual_request_headers,
                    request_body: body,
                    response_status: status.as_u16(),
                    response_headers,
                    response_body: Some(response_body),
                    validations,
                    duration_ms: start.elapsed().as_millis() as u64,
                    error: None,
                }
            }
            Err(e) => ExecutionResult {
                success: false,
                step_name: step_name.to_string(),
                request_url,
                request_method: method,
                request_headers: headers,
                request_body: body,
                response_status: 0,
                response_headers: HashMap::new(),
                response_body: None,
                validations: vec![],
                duration_ms: start.elapsed().as_millis() as u64,
                error: Some(e),
            },
        }
    }

    /// 路径映射转换
    fn transform_path(&self, path: &str) -> String {
        for (source, target) in &self.path_mapping {
            if path.starts_with(source) {
                return path.replacen(source, target, 1);
            }
        }
        path.to_string()
    }

    /// 替换字符串中的变量
    fn replace_variables(&self, text: &str, variables: &HashMap<String, JsonValue>) -> String {
        let re = Regex::new(r"\{([^}]+)\}").unwrap();
        
        re.replace_all(text, |caps: &regex::Captures| {
            let var_path = &caps[1];
            
            // 支持嵌套访问 {user.name}
            let parts: Vec<&str> = var_path.split('.').collect();
            
            let mut value: Option<&JsonValue> = variables.get(parts[0]);
            
            for part in parts.iter().skip(1) {
                if let Some(v) = value {
                    value = v.get(part);
                } else {
                    break;
                }
            }
            
            match value {
                Some(v) => json_value_to_string(v),
                None => caps[0].to_string(),
            }
        }).to_string()
    }

    /// 替换 Map 中的变量
    fn replace_map_variables(
        &self,
        map: &HashMap<String, YamlValue>,
        variables: &HashMap<String, JsonValue>,
    ) -> HashMap<String, JsonValue> {
        map.iter()
            .map(|(k, v)| {
                let json_v = yaml_to_json(v);
                (k.clone(), self.replace_json_variables(&json_v, variables))
            })
            .collect()
    }

    /// 替换 JSON 中的变量
    fn replace_json_variables(
        &self,
        value: &JsonValue,
        variables: &HashMap<String, JsonValue>,
    ) -> JsonValue {
        match value {
            JsonValue::String(s) => {
                JsonValue::String(self.replace_variables(s, variables))
            }
            JsonValue::Array(arr) => {
                JsonValue::Array(
                    arr.iter()
                        .map(|v| self.replace_json_variables(v, variables))
                        .collect()
                )
            }
            JsonValue::Object(obj) => {
                JsonValue::Object(
                    obj.iter()
                        .map(|(k, v)| (k.clone(), self.replace_json_variables(v, variables)))
                        .collect()
                )
            }
            _ => value.clone(),
        }
    }

    /// 验证响应
    fn validate_response(
        &self,
        response: &JsonValue,
        validations: &[Validation],
    ) -> Vec<ValidationResult> {
        validations.iter().map(|v| {
            let actual = get_json_field(response, &v.field);
            let expected_json = yaml_to_json(&v.expected);
            
            let passed = match v.operator.as_str() {
                "equals" => actual.as_ref() == Some(&expected_json),
                "exists" => actual.is_some(),
                "not_empty" => {
                    actual.as_ref().map(|a| !is_empty(a)).unwrap_or(false)
                }
                "contains" => {
                    if let (Some(actual_val), JsonValue::String(expected_str)) = (&actual, &expected_json) {
                        json_value_to_string(actual_val).contains(expected_str)
                    } else {
                        false
                    }
                }
                _ => false,
            };

            ValidationResult {
                field: v.field.clone(),
                operator: v.operator.clone(),
                expected: json_value_to_string(&expected_json),
                actual: actual.map(|a| json_value_to_string(&a)).unwrap_or_else(|| "null".to_string()),
                passed,
            }
        }).collect()
    }
}

/// YAML Value 转 JSON Value
fn yaml_to_json(yaml: &YamlValue) -> JsonValue {
    serde_json::to_value(yaml).unwrap_or(JsonValue::Null)
}

/// JSON Value 转字符串
fn json_value_to_string(value: &JsonValue) -> String {
    match value {
        JsonValue::String(s) => s.clone(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Null => "null".to_string(),
        _ => serde_json::to_string(value).unwrap_or_default(),
    }
}

/// 获取 JSON 嵌套字段
fn get_json_field(value: &JsonValue, path: &str) -> Option<JsonValue> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;
    
    for part in parts {
        current = current.get(part)?;
    }
    
    Some(current.clone())
}

/// 判断值是否为空
fn is_empty(value: &JsonValue) -> bool {
    match value {
        JsonValue::Null => true,
        JsonValue::String(s) => s.is_empty(),
        JsonValue::Array(a) => a.is_empty(),
        JsonValue::Object(o) => o.is_empty(),
        _ => false,
    }
}
