//! 应用数据持久化存储
//! 
//! 使用系统应用数据目录存储配置和历史记录

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use chrono::{DateTime, Utc};

/// 应用数据目录名
const APP_DIR: &str = "yuml-ddt";
/// 应用数据文件名
const APP_DATA_FILE: &str = "app_data.json";

/// 最近打开的工作区记录
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentWorkspace {
    /// 工作区路径
    pub path: String,
    /// 工作区名称（目录名）
    pub name: String,
    /// 最后打开时间
    pub last_opened: DateTime<Utc>,
}

/// 应用设置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppSettings {
    /// 最后使用的工作区路径
    #[serde(default)]
    pub last_workspace: Option<String>,
    /// 编辑器字体大小
    #[serde(default = "default_font_size")]
    pub editor_font_size: u32,
    /// 主题 (dark/light)
    #[serde(default = "default_theme")]
    pub theme: String,
}

fn default_font_size() -> u32 { 13 }
fn default_theme() -> String { "dark".to_string() }

/// 完整的应用数据
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppData {
    /// 最近打开的工作区列表
    #[serde(default)]
    pub recent_workspaces: Vec<RecentWorkspace>,
    /// 应用设置
    #[serde(default)]
    pub settings: AppSettings,
}

impl AppData {
    /// 获取应用数据目录
    fn get_data_dir() -> Option<PathBuf> {
        dirs::data_dir().map(|p| p.join(APP_DIR))
    }

    /// 获取数据文件路径
    fn get_data_file_path() -> Option<PathBuf> {
        Self::get_data_dir().map(|p| p.join(APP_DATA_FILE))
    }

    /// 加载应用数据
    pub fn load() -> Self {
        if let Some(path) = Self::get_data_file_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(data) = serde_json::from_str(&content) {
                        return data;
                    }
                }
            }
        }
        AppData::default()
    }

    /// 保存应用数据
    pub fn save(&self) -> Result<(), String> {
        let data_dir = Self::get_data_dir()
            .ok_or_else(|| "无法获取应用数据目录".to_string())?;
        
        // 确保目录存在
        if !data_dir.exists() {
            fs::create_dir_all(&data_dir)
                .map_err(|e| format!("创建数据目录失败: {}", e))?;
        }

        let file_path = data_dir.join(APP_DATA_FILE);
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("序列化数据失败: {}", e))?;
        
        fs::write(&file_path, content)
            .map_err(|e| format!("写入数据文件失败: {}", e))?;

        Ok(())
    }

    /// 添加最近打开的工作区
    pub fn add_recent_workspace(&mut self, path: &str) {
        // 提取目录名作为名称
        let name = std::path::Path::new(path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("未知")
            .to_string();

        // 移除已存在的相同路径
        self.recent_workspaces.retain(|w| w.path != path);

        // 添加到开头
        self.recent_workspaces.insert(0, RecentWorkspace {
            path: path.to_string(),
            name,
            last_opened: Utc::now(),
        });

        // 只保留最近 10 个
        if self.recent_workspaces.len() > 10 {
            self.recent_workspaces.truncate(10);
        }

        // 更新最后使用的工作区
        self.settings.last_workspace = Some(path.to_string());
    }

    /// 移除最近工作区记录
    pub fn remove_recent_workspace(&mut self, path: &str) {
        self.recent_workspaces.retain(|w| w.path != path);
    }

    /// 清空最近工作区列表
    pub fn clear_recent_workspaces(&mut self) {
        self.recent_workspaces.clear();
    }
}

// ============ Tauri 命令 ============

/// 获取应用数据
#[tauri::command]
pub fn get_app_data() -> AppData {
    AppData::load()
}

/// 保存应用数据
#[tauri::command]
pub fn save_app_data(data: AppData) -> Result<(), String> {
    data.save()
}

/// 添加最近工作区
#[tauri::command]
pub fn add_recent_workspace(path: String) -> Result<AppData, String> {
    let mut data = AppData::load();
    data.add_recent_workspace(&path);
    data.save()?;
    Ok(data)
}

/// 移除最近工作区
#[tauri::command]
pub fn remove_recent_workspace(path: String) -> Result<AppData, String> {
    let mut data = AppData::load();
    data.remove_recent_workspace(&path);
    data.save()?;
    Ok(data)
}

/// 清空最近工作区
#[tauri::command]
pub fn clear_recent_workspaces() -> Result<AppData, String> {
    let mut data = AppData::load();
    data.clear_recent_workspaces();
    data.save()?;
    Ok(data)
}

/// 获取最近工作区列表
#[tauri::command]
pub fn get_recent_workspaces() -> Vec<RecentWorkspace> {
    AppData::load().recent_workspaces
}

/// 更新应用设置
#[tauri::command]
pub fn update_settings(settings: AppSettings) -> Result<(), String> {
    let mut data = AppData::load();
    data.settings = settings;
    data.save()
}

/// 获取应用设置
#[tauri::command]
pub fn get_settings() -> AppSettings {
    AppData::load().settings
}
