//! HTTP 客户端封装

use reqwest::{Client, StatusCode};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;

use crate::cache::Cache;
use crate::config::AuthConfig;

pub struct HttpClient {
    client: Client,
    base_url: String,
    auth_config: Option<AuthConfig>,
    cache: Cache,
    debug: bool,
}

impl HttpClient {
    pub fn new(base_url: String, auth_config: Option<AuthConfig>, debug: bool) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            base_url,
            auth_config,
            cache: Cache::new(),
            debug,
        }
    }

    /// 获取 token
    async fn get_token(&self) -> Result<Option<String>, String> {
        let Some(auth) = self.auth_config.as_ref() else {
            return Ok(None);
        };
        
        if auth.token_url.is_empty() {
            return Ok(None);
        }

        let token_field = if auth.token_field.is_empty() {
            "access_token"
        } else {
            &auth.token_field
        };

        // 先检查缓存
        let cache_key = if auth.token_cache_key.is_empty() {
            "default_token"
        } else {
            &auth.token_cache_key
        };

        if let Some(cached) = self.cache.get(cache_key) {
            if let Some(token) = extract_token(&cached, token_field) {
                return Ok(Some(token));
            }
        }

        // 请求新 token
        let body = auth.request_body.clone().unwrap_or_else(|| {
            HashMap::from([
                ("client_id".to_string(), Value::String(auth.client_id.clone())),
                ("grant_type".to_string(), Value::String(auth.grant_type.clone())),
                ("username".to_string(), Value::String(auth.username.clone())),
                ("password".to_string(), Value::String(auth.password.clone())),
            ])
        });
        let request = self.client.post(&auth.token_url);
        let request = match auth.request_format.as_str() {
            "" | "form" => {
                let form: Result<HashMap<_, _>, String> = body.iter().map(|(key, value)| {
                    let value = match value {
                        Value::String(value) => value.clone(),
                        Value::Bool(_) | Value::Number(_) => value.to_string(),
                        _ => return Err("auth.request_body form values must be strings, numbers or booleans".to_string()),
                    };
                    Ok((key.clone(), value))
                }).collect();
                request.form(&form?)
            }
            "json" => request.json(&body),
            _ => return Err("auth.request_format must be form or json".to_string()),
        };
        let response = request.send().await
            .map_err(|_| "Token request failed".to_string())?;
        if !response.status().is_success() {
            return Err(format!("Token request failed: HTTP {}", response.status()));
        }
        let token_data: Value = response.json().await
            .map_err(|_| "Token response is not valid JSON".to_string())?;
        let token = extract_token(&token_data, token_field)
            .ok_or_else(|| format!("Token response field '{}' must be a non-empty string", token_field))?;
        self.cache.set(cache_key, token_data, Some(3600));
        Ok(Some(token))
    }

    /// 发送 GET 请求
    pub async fn get(
        &self,
        path: &str,
        params: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<(StatusCode, HashMap<String, String>, HashMap<String, String>, Value), String> {
        let url = format!("{}{}", self.base_url, path);
        
        let mut request = self.client.get(&url);
        
        if let Some(p) = params {
            request = request.query(p);
        }
        
        let (request, request_headers) = self.apply_headers(request, headers).await?;
        
        self.execute_request(request, request_headers, &url, "GET").await
    }

    /// 发送 POST 请求
    pub async fn post(
        &self,
        path: &str,
        body: Option<&Value>,
        params: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<(StatusCode, HashMap<String, String>, HashMap<String, String>, Value), String> {
        let url = format!("{}{}", self.base_url, path);
        
        let mut request = self.client.post(&url);
        
        if let Some(p) = params {
            request = request.query(p);
        }
        
        let has_body = body.is_some();
        if let Some(b) = body {
            request = request.json(b);
        }
        
        let (request, mut request_headers) = self.apply_headers(request, headers).await?;
        
        // 如果使用了 .json()，记录 Content-Type (reqwest 会自动添加)
        if has_body && !request_headers.contains_key("Content-Type") && !request_headers.contains_key("content-type") {
            request_headers.insert("content-type".to_string(), "application/json".to_string());
        }
        
        self.execute_request(request, request_headers, &url, "POST").await
    }

    /// 发送 PUT 请求
    pub async fn put(
        &self,
        path: &str,
        body: Option<&Value>,
        params: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<(StatusCode, HashMap<String, String>, HashMap<String, String>, Value), String> {
        let url = format!("{}{}", self.base_url, path);
        
        let mut request = self.client.put(&url);
        
        if let Some(p) = params {
            request = request.query(p);
        }
        
        let has_body = body.is_some();
        if let Some(b) = body {
            request = request.json(b);
        }
        
        let (request, mut request_headers) = self.apply_headers(request, headers).await?;
        
        // 如果使用了 .json()，记录 Content-Type (reqwest 会自动添加)
        if has_body && !request_headers.contains_key("Content-Type") && !request_headers.contains_key("content-type") {
            request_headers.insert("content-type".to_string(), "application/json".to_string());
        }
        
        self.execute_request(request, request_headers, &url, "PUT").await
    }

    /// 发送 DELETE 请求
    pub async fn delete(
        &self,
        path: &str,
        params: Option<&HashMap<String, String>>,
        headers: Option<&HashMap<String, String>>,
    ) -> Result<(StatusCode, HashMap<String, String>, HashMap<String, String>, Value), String> {
        let url = format!("{}{}", self.base_url, path);
        
        let mut request = self.client.delete(&url);
        
        if let Some(p) = params {
            request = request.query(p);
        }
        
        let (request, request_headers) = self.apply_headers(request, headers).await?;
        
        self.execute_request(request, request_headers, &url, "DELETE").await
    }

    /// 应用请求头和认证，返回 (request, 完整的请求头)
    async fn apply_headers(
        &self,
        mut request: reqwest::RequestBuilder,
        custom_headers: Option<&HashMap<String, String>>,
    ) -> Result<(reqwest::RequestBuilder, HashMap<String, String>), String> {
        let mut all_headers: HashMap<String, String> = HashMap::new();
        
        // 添加自定义 headers
        if let Some(headers) = custom_headers {
            for (key, value) in headers {
                request = request.header(key, value);
                all_headers.insert(key.clone(), value.clone());
            }
        }
        
        // 添加认证 token
        if let Some(token) = self.get_token().await? {
            if let Some(auth) = &self.auth_config {
                let auth_position = if auth.auth_position.is_empty() {
                    "header"
                } else {
                    &auth.auth_position
                };
                
                match auth_position {
                    "header" => {
                        let key = if auth.auth_key.is_empty() {
                            "Authorization"
                        } else {
                            &auth.auth_key
                        };
                        let prefix = if auth.auth_prefix.is_empty() {
                            "Bearer"
                        } else {
                            &auth.auth_prefix
                        };
                        let value = format!("{} {}", prefix.trim(), token);
                        request = request.header(key, value.clone());
                        all_headers.insert(key.to_string(), value);
                    }
                    "query" => {
                        let key = if auth.auth_key.is_empty() {
                            "access_token"
                        } else {
                            &auth.auth_key
                        };
                        request = request.query(&[(key, &token)]);
                        // query 参数不加到 headers 中
                    }
                    _ => {}
                }
            }
        }
        
        Ok((request, all_headers))
    }

    /// 执行请求
    async fn execute_request(
        &self,
        request: reqwest::RequestBuilder,
        request_headers: HashMap<String, String>,
        url: &str,
        method: &str,
    ) -> Result<(StatusCode, HashMap<String, String>, HashMap<String, String>, Value), String> {
        if self.debug {
            println!("🔍 [DEBUG] {} {}", method, url);
        }

        let response = request.send().await.map_err(|e| e.to_string())?;
        let status = response.status();
        
        // 提取响应头
        let response_headers: HashMap<String, String> = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();
        
        let body: Value = response.json().await.unwrap_or(Value::Null);
        
        if self.debug {
            println!("🔍 [DEBUG] Status: {}", status);
            println!("🔍 [DEBUG] Response: {}", serde_json::to_string_pretty(&body).unwrap_or_default());
        }
        
        Ok((status, request_headers, response_headers, body))
    }
}

fn extract_token(data: &Value, field: &str) -> Option<String> {
    let mut value = data;
    for part in field.split('.') {
        value = value.get(part)?;
    }
    value.as_str().filter(|token| !token.trim().is_empty()).map(str::to_owned)
}
