//! 本地缓存实现

use serde_json::Value;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// 缓存条目
#[derive(Debug, Clone)]
struct CacheEntry {
    value: Value,
    expire_time: Option<u64>,
}

/// 简单的内存缓存
pub struct Cache {
    data: RwLock<HashMap<String, CacheEntry>>,
}

impl Cache {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }

    /// 获取当前时间戳
    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// 设置缓存
    pub fn set(&self, key: &str, value: Value, ttl_seconds: Option<u64>) {
        let expire_time = ttl_seconds.map(|ttl| Self::now() + ttl);
        
        let entry = CacheEntry {
            value,
            expire_time,
        };
        
        if let Ok(mut data) = self.data.write() {
            data.insert(key.to_string(), entry);
        }
    }

    /// 获取缓存
    pub fn get(&self, key: &str) -> Option<Value> {
        if let Ok(data) = self.data.read() {
            if let Some(entry) = data.get(key) {
                // 检查是否过期
                if let Some(expire_time) = entry.expire_time {
                    if Self::now() > expire_time {
                        return None;
                    }
                }
                return Some(entry.value.clone());
            }
        }
        None
    }

    /// 删除缓存
    pub fn remove(&self, key: &str) {
        if let Ok(mut data) = self.data.write() {
            data.remove(key);
        }
    }

    /// 清空缓存
    pub fn clear(&self) {
        if let Ok(mut data) = self.data.write() {
            data.clear();
        }
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new()
    }
}
