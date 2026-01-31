//! Tauri 命令定义

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::Value as JsonValue;

use crate::config::{ExecutionResult, StepInfo};
use crate::engine::TestEngine;

/// 列出目录下的 YAML 文件
#[tauri::command]
pub fn list_yaml_files(directory: String) -> Result<Vec<String>, String> {
    let path = Path::new(&directory);
    
    if !path.exists() {
        return Err(format!("目录不存在: {}", directory));
    }
    
    let mut files = Vec::new();
    
    fn scan_dir(dir: &Path, files: &mut Vec<String>) -> Result<(), String> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let path = entry.path();
                
                if path.is_dir() {
                    scan_dir(&path, files)?;
                } else if let Some(ext) = path.extension() {
                    if ext == "yml" || ext == "yaml" {
                        if let Some(path_str) = path.to_str() {
                            files.push(path_str.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    scan_dir(path, &mut files)?;
    files.sort();
    
    Ok(files)
}

/// 读取 YAML 文件内容
#[tauri::command]
pub fn read_yaml_file(file_path: String) -> Result<String, String> {
    fs::read_to_string(&file_path)
        .map_err(|e| format!("读取文件失败: {}", e))
}

/// 保存 YAML 文件
#[tauri::command]
pub fn save_yaml_file(file_path: String, content: String) -> Result<(), String> {
    fs::write(&file_path, content)
        .map_err(|e| format!("保存文件失败: {}", e))
}

/// 解析 YAML 文件中的 steps
#[tauri::command]
pub fn parse_yaml_steps(content: String) -> Result<Vec<StepInfo>, String> {
    let engine = TestEngine::from_yaml(&content)?;
    Ok(engine.get_step_list())
}

/// 获取配置文件的 step 列表
#[tauri::command]
pub fn get_step_list(file_path: String) -> Result<Vec<StepInfo>, String> {
    let engine = TestEngine::from_file(&file_path)?;
    Ok(engine.get_step_list())
}

/// 执行单个 step
#[tauri::command]
pub async fn execute_step(
    file_path: String,
    step_name: String,
    variables: Option<HashMap<String, JsonValue>>,
) -> Result<ExecutionResult, String> {
    let mut engine = TestEngine::from_file(&file_path)?;
    Ok(engine.execute_step(&step_name, variables).await)
}
