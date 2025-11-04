use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use base64::Engine;
use serde::Serialize;
use tauri_plugin_shell::ShellExt;

use crate::instance_cfg::parse_general;

pub mod instance_cfg;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Serialize, Clone)]
struct Instance {
    id: String,
    name: String,
    icon_b64: Option<String>,
    last_launch_ms: Option<i64>,
    total_time_ms: Option<i64>,
}

#[derive(Serialize)]
struct ScanResult {
    instances: Vec<Instance>,
    groups: HashMap<String, Vec<String>>,
}

#[tauri::command]
fn scan_prism(root_override: Option<String>) -> Result<ScanResult, String> {
    scan(root_override).map_err(|e| e.to_string())
}

#[tauri::command]
fn launch_instance(
    app_handle: tauri::AppHandle,
    instance_id: String,
    root_override: Option<String>,
) -> Result<(), String> {
    launch(&app_handle, &instance_id, root_override.as_deref()).map_err(|e| e.to_string())
}

#[tauri::command]
fn show_instance(
    app_handle: tauri::AppHandle,
    instance_id: String,
    root_override: Option<String>,
) -> Result<(), String> {
    show_in_prism(&app_handle, &instance_id, root_override.as_deref()).map_err(|e| e.to_string())
}

fn default_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".local/share/PrismLauncher")
}

fn instances_dir(root: &Path) -> PathBuf {
    root.join("instances")
}

fn icons_dir(root: &Path) -> PathBuf {
    root.join("icons")
}

fn scan(root_override: Option<String>) -> anyhow::Result<ScanResult> {
    let root = root_override
        .map(PathBuf::from)
        .unwrap_or_else(default_root);

    let inst_dir = instances_dir(&root);
    let mut instances = vec![];

    for ent in std::fs::read_dir(&inst_dir).unwrap_or_else(|_| std::fs::read_dir(".").unwrap()) {
        let ent = match ent {
            Ok(e) => e,
            Err(_) => continue,
        };

        if ent.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let id = ent.file_name().to_string_lossy().to_string();
            let cfg = ent.path().join("instance.cfg");
            let (name, icon_key, last, total) = parse_general(&cfg)?;
            let icon_b64 = resolve_icon_base64(&root, &ent.path(), icon_key.as_deref());
            instances.push(Instance {
                name: name.unwrap_or_else(|| id.clone()),
                id,
                icon_b64,
                last_launch_ms: last,
                total_time_ms: total,
            });
        }
    }

    let mut groups: HashMap<String, Vec<String>> = HashMap::new();
    let groups_path = inst_dir.join("instgroups.json");
    if let Ok(bytes) = std::fs::read(&groups_path) {
        #[derive(serde::Deserialize)]
        struct GroupsFile {
            #[serde(rename = "groups")]
            groups: HashMap<String, Group>,
        }
        #[derive(serde::Deserialize)]
        struct Group {
            instances: Vec<String>,
            hidden: Option<bool>,
        }
        if let Ok(parsed) = serde_json::from_slice::<serde_json::Value>(&bytes) {
            if let Some(obj) = parsed.get("groups").and_then(|g| g.as_object()) {
                for (name, val) in obj.iter() {
                    let ids = val
                        .get("instances")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|s| s.as_str().map(|x| x.to_string()))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                    groups.insert(name.clone(), ids);
                }
            }
        }
    }

    // recent first
    instances.sort_by_key(|i| std::cmp::Reverse(i.last_launch_ms.unwrap_or(0)));

    Ok(ScanResult { instances, groups })
}

fn resolve_icon(root: &Path, instance_dir: &Path, icon_key: Option<&str>) -> Option<PathBuf> {
    let possible_formats = ["png", "jpg", "jpeg", "webp", "gif", "svg"];

    if let Some(key) = icon_key {
        for ext in possible_formats {
            let p = icons_dir(root).join(format!("{key}.{ext}"));
            if p.exists() {
                return Some(p);
            }
        }
    }
    let inst_icon = instance_dir.join("icon.png");
    if inst_icon.exists() {
        return Some(inst_icon);
    }
    None
}

fn resolve_icon_base64(root: &Path, inst_path: &Path, icon_key: Option<&str>) -> Option<String> {
    if let Some(icon_path) = resolve_icon(root, inst_path, icon_key) {
        if let Ok(bytes) = std::fs::read(&icon_path) {
            return Some(format!(
                "data:image/{ext};base64,{b64}",
                ext = icon_path
                    .extension()
                    .and_then(|s| s.to_str())
                    .unwrap_or("png"),
                b64 = base64::engine::general_purpose::STANDARD.encode(&bytes)
            ));
        }
    }

    None
}

fn launch(
    app_handle: &tauri::AppHandle,
    instance_id: &str,
    root_override: Option<&str>,
) -> anyhow::Result<()> {
    let shell = app_handle.shell();
    let mut cmd = shell.command("prismlauncher");
    if let Some(root) = root_override {
        cmd = cmd.arg("--dir").arg(root);
    }
    cmd = cmd.arg("--launch").arg(instance_id);
    cmd.spawn()?;

    Ok(())
}

fn show_in_prism(
    app_handle: &tauri::AppHandle,
    instance_id: &str,
    root_override: Option<&str>,
) -> anyhow::Result<()> {
    let shell = app_handle.shell();
    let mut cmd = shell.command("prismlauncher");
    if let Some(root) = root_override {
        cmd = cmd.arg("--dir").arg(root);
    }
    cmd = cmd.arg("--show").arg(instance_id);
    cmd.spawn()?;

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            scan_prism,
            launch_instance,
            show_instance
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
