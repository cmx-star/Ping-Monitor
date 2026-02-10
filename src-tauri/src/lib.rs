mod monitor;

use monitor::{Monitor, DisplayRule, HostConfig, HostPreset};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{Emitter, State, Manager};
use tokio::sync::Mutex;
use uuid::Uuid;
use std::fs;
use std::path::PathBuf;
use reqwest;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppSettings {
    pub hosts: Vec<HostConfig>,
    pub presets: Vec<HostPreset>,
    pub ping_interval: u64,
    pub auto_start: bool,
    pub notification_type: String, // "system" | "bark"
    pub bark_url: String,
    pub display_strategy: String, // "mean" | "worst" | "fastest" | "first"
    pub show_latency: bool,
    pub show_labels: bool,
    pub log_level: String, // "debug" | "info" | "warn" | "error"
    pub enable_notifications: bool,
}



#[derive(Clone)]
struct AppState {
    monitors: Arc<Mutex<HashMap<Uuid, Arc<Monitor>>>>,
    settings: Arc<Mutex<AppSettings>>,
    tray_cache: Arc<Mutex<HashMap<Uuid, monitor::PingStats>>>,
    is_visible_flag: Arc<std::sync::atomic::AtomicBool>,
    last_click: Arc<std::sync::Mutex<std::time::Instant>>,
}

impl AppState {
    fn get_settings_path(app: &tauri::AppHandle) -> PathBuf {
        app.path().app_data_dir().unwrap().join("settings.json")
    }

    async fn save_settings(&self, app: &tauri::AppHandle) -> Result<(), String> {
        let settings = self.settings.lock().await;
        let path = Self::get_settings_path(app);
        fs::create_dir_all(path.parent().unwrap()).map_err(|e| e.to_string())?;
        let json = serde_json::to_string_pretty(&*settings).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
        
        // Trigger tray update on settings change
        let tray_cache = self.tray_cache.lock().await.clone();
        Self::update_tray_title(app, &settings, &tray_cache);
        
        Ok(())
    }
    
    fn update_tray_title(app: &tauri::AppHandle, settings: &AppSettings, cache: &HashMap<Uuid, monitor::PingStats>) {
        if let Some(tray) = app.tray_by_id("main-tray") {
            if cache.is_empty() {
                let _ = tray.set_title(Some(" Initializing..."));
                return;
            }
            
            // Filter hosts that are currently in the cache (implies they are running/have data)
            // and apply strategy
            let mut active_stats: Vec<&monitor::PingStats> = cache.values().collect();
            
            // Sort based on strategy
            // "first" -> order by settings.hosts order
            // "best" -> lowest latency
            // "worst" -> highest latency
            // "mean" -> average of all (not single host) or maybe "average" strategy means show average latency? 
            // Checking Swift code: 
            // .average => calculates average of all active hosts
            // .best => min latency
            // .worst => max latency or first unreachable
            // .first => first in list
            
            let target_stat: Option<monitor::PingStats> = match settings.display_strategy.as_str() {
                "mean" => {
                    let count = active_stats.len();
                    if count > 0 {
                        let sum: f64 = active_stats.iter().map(|s| s.current).sum();
                        let avg = sum / count as f64;
                        // Construct a dummy stat for average
                        let mut dummy = active_stats[0].clone();
                        dummy.current = avg;
                        dummy.labels = vec!["AVG".to_string()];
                        Some(dummy)
                    } else {
                        None
                    }
                },
                "worst" => {
                    // Swift: if any unreachable, show that. Else max latency.
                    // We don't have explicit "is_reachable" in stats, but status might help?
                    // For now just sort by latency desc
                    active_stats.sort_by(|a, b| b.current.partial_cmp(&a.current).unwrap_or(std::cmp::Ordering::Equal));
                    active_stats.first().map(|s| (*s).clone())
                }
                "fastest" => {
                   active_stats.sort_by(|a, b| a.current.partial_cmp(&b.current).unwrap_or(std::cmp::Ordering::Equal));
                   active_stats.first().map(|s| (*s).clone()) 
                }
                _ => { // "first" or default
                    // Need to find which stat corresponds to the first configured host
                    let first_id = settings.hosts.first().map(|h| h.id);
                    if let Some(fid) = first_id {
                        cache.get(&fid).cloned()
                    } else {
                        active_stats.first().map(|s| (*s).clone())
                    }
                }
            };
            
            if let Some(stat) = target_stat {
                let mut parts = Vec::new();
                
                if settings.show_latency {
                    parts.push(format!("{}ms", stat.current as u64));
                }
                
                if settings.show_labels {
                    for label in &stat.labels {
                        parts.push(label.clone());
                    }
                }
                
                // Fallback if both hidden
                if parts.is_empty() {
                    parts.push("Running".to_string());
                }
                
                let title = format!(" {}", parts.join(" "));
                let _ = tray.set_title(Some(title));
            } else {
                 let _ = tray.set_title(Some(" No Data"));
            }
        }
    }
}

async fn send_notification(
    title: &str,
    body: &str,
    notification_type: &str,
    bark_url: &str,
    app: &tauri::AppHandle,
) {
    if notification_type == "bark" && !bark_url.is_empty() {
        let url = format!("{}/{}/{}", bark_url.trim_end_matches('/'), title, body);
        let _ = reqwest::get(url).await;
    } else {
        // Native tauri notification
        use tauri_plugin_notification::NotificationExt;
        app.notification()
            .builder()
            .title(title)
            .body(body)
            .show()
            .unwrap();
    }
}

#[tauri::command]
async fn start_monitoring(
    host_id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&host_id).map_err(|e| e.to_string())?;
    
    // Cleanup existing monitor if present
    {
        let mut monitors = state.monitors.lock().await;
        if let Some(existing) = monitors.remove(&uuid) {
            println!("[Rust] Removing existing monitor for replacement: {}", uuid);
            existing.stop();
        }
    }

    let settings = state.settings.lock().await.clone();
    let host = settings.hosts.iter().find(|h| h.id == uuid).ok_or("Host not found")?.clone();

    let mut monitors = state.monitors.lock().await;
    // No need to remove again, we just did cleanup above
    
    // Resolve log path to App Data directory
    let app_data_dir = app.path().app_data_dir().unwrap();
    let log_dir = app_data_dir.join("logs");
    if !log_dir.exists() {
        std::fs::create_dir_all(&log_dir).map_err(|e| e.to_string())?;
    }
    let log_path = log_dir.join(format!("ping_{}.csv", uuid));
    let log_path_str = log_path.to_str().unwrap().to_string();

    let (monitor, mut rx) = Monitor::new(
        uuid,
        &host.address, 
        200.0, 
        &log_path_str,
        host.display_rules.clone(),
        settings.ping_interval
    );
    let app_clone = app.clone();
    let state_clone = state.inner().clone(); // Clone internal Arc-holding struct
    let notification_type = settings.notification_type.clone();
    let bark_url = settings.bark_url.clone();
    let host_name = host.name.clone();

    println!("[Rust] About to spawn event loop for {}", host_name);

    let consumer_task = tokio::spawn(async move {
        println!("[Rust] Starting event loop for host: {}", host_name);
        let mut last_latency = 0.0;
        loop {
            match rx.recv().await {
                Ok(stats) => {
                    if let Err(e) = app_clone.emit("ping-stats", stats.clone()) {
                        eprintln!("[Rust] Failed to emit stats for {}: {}", host_name, e);
                    }
                    
                    // Update cache and Tray
                    {
                        let mut cache = state_clone.tray_cache.lock().await;
                        cache.insert(stats.host_id, stats.clone());
                    }
                    // Re-read settings for latest display strategy
                    let current_settings = state_clone.settings.lock().await;
                    let current_cache = state_clone.tray_cache.lock().await;
                    AppState::update_tray_title(&app_clone, &current_settings, &current_cache);

                    // Notification logic parity: 
                    // If latency > 100ms or status changes to bad
                    if stats.current > 100.0 && last_latency <= 100.0 {
                        send_notification(
                            "⚠️ 延迟过高",
                            &format!("{}: {:.1}ms", host_name, stats.current),
                            &notification_type,
                            &bark_url,
                            &app_clone
                        ).await;
                    }
                    last_latency = stats.current;
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(n)) => {
                    eprintln!("[Rust] Event loop lagged by {} for {}", n, host_name);
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    println!("[Rust] Event loop closed for {}", host_name);
                    
                    // Remove from cache when closed
                     {
                        let mut cache = state_clone.tray_cache.lock().await;
                        cache.remove(&uuid);
                    }
                    // Update tray
                    let current_settings = state_clone.settings.lock().await;
                    let current_cache = state_clone.tray_cache.lock().await;
                     AppState::update_tray_title(&app_clone, &current_settings, &current_cache);
                    break;
                }
            }
        }
    });

    // Register the consumer task to be aborted when monitor stops
    monitor.add_abort_handle(consumer_task.abort_handle());

    println!("[Rust] Event loop spawned. Starting monitor...");
    monitor.clone().start().await.map_err(|e| e.to_string())?;
    println!("[Rust] Monitor started.");

    monitors.insert(uuid, monitor);
    Ok(())
}

#[tauri::command]
async fn stop_monitoring(
    host_id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&host_id).map_err(|e| e.to_string())?;
    let mut monitors = state.monitors.lock().await;
    if let Some(monitor) = monitors.remove(&uuid) {
        monitor.stop();
    }
    Ok(())
}

#[tauri::command]
async fn add_host(
    config: HostConfig,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    {
        let mut settings = state.settings.lock().await;
        settings.hosts.push(config);
    }
    state.save_settings(&app).await
}

#[tauri::command]
async fn remove_host(
    host_id: String,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let uuid = Uuid::parse_str(&host_id).map_err(|e| e.to_string())?;
    {
        let mut settings = state.settings.lock().await;
        settings.hosts.retain(|h| h.id != uuid);
    }
    state.save_settings(&app).await
}

#[tauri::command]
async fn update_host(
    config: HostConfig,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    {
        let mut settings = state.settings.lock().await;
        if let Some(h) = settings.hosts.iter_mut().find(|h| h.id == config.id) {
            *h = config;
        }
    }
    state.save_settings(&app).await
}

#[tauri::command]
async fn apply_settings(
    new_settings: AppSettings,
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    {
        let mut settings = state.settings.lock().await;
        *settings = new_settings;
    }
    state.save_settings(&app).await
}

#[tauri::command]
async fn start_all(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let hosts = {
        let settings = state.settings.lock().await;
        settings.hosts.clone()
    };
    
    for host in hosts {
        let _ = start_monitoring(host.id.to_string(), state.clone(), app.clone()).await;
    }
    Ok(())
}

#[tauri::command]
async fn stop_all(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut monitors = state.monitors.lock().await;
    for (_, monitor) in monitors.iter() {
        monitor.stop();
    }
    monitors.clear();
    Ok(())
}

#[tauri::command]
async fn get_hosts(state: State<'_, AppState>) -> Result<Vec<HostConfig>, String> {
    let settings = state.settings.lock().await;
    Ok(settings.hosts.clone())
}

#[tauri::command]
async fn get_settings(state: State<'_, AppState>) -> Result<AppSettings, String> {
    let settings = state.settings.lock().await;
    Ok(settings.clone())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .setup(|app| {
            let app_handle = app.handle().clone();
            
            // 1. Prepare Settings & State FIRST
            let settings_path = AppState::get_settings_path(&app_handle);
            
            let initial_settings = if settings_path.exists() {
                let data = fs::read_to_string(settings_path).unwrap();
                serde_json::from_str(&data).unwrap_or(AppSettings {
                    hosts: vec![],
                    ping_interval: 1,
                    auto_start: false,
                    notification_type: "system".to_string(),
                    bark_url: "".to_string(),
                    display_strategy: "first".to_string(),
                    show_latency: true,
                    show_labels: true,
                    log_level: "info".to_string(),
                    enable_notifications: true,
                    presets: vec![],
                })
            } else {
                AppSettings {
                    hosts: vec![
                        HostConfig {
                            id: Uuid::new_v4(),
                            name: "Google DNS".to_string(),
                            address: "8.8.8.8".to_string(),
                            command: "".to_string(),
                            display_rules: vec![
                                DisplayRule { id: Uuid::new_v4(), condition: "less".to_string(), threshold: 50.0, label: "P2P".to_string(), enabled: true },
                                DisplayRule { id: Uuid::new_v4(), condition: "greater".to_string(), threshold: 50.0, label: "转发".to_string(), enabled: true },
                            ],
                        }
                    ],
                    ping_interval: 5,
                    auto_start: false,
                    notification_type: "system".to_string(),
                    bark_url: "".to_string(),
                    display_strategy: "first".to_string(),
                    show_latency: true,
                    show_labels: true,
                    log_level: "info".to_string(),
                    enable_notifications: true,
                    presets: vec![
                        HostPreset { id: Uuid::new_v4(), name: "Google DNS".to_string(), address: "8.8.8.8".to_string(), command: "".to_string() },
                        HostPreset { id: Uuid::new_v4(), name: "Cloudflare".to_string(), address: "1.1.1.1".to_string(), command: "".to_string() },
                        HostPreset { id: Uuid::new_v4(), name: "Baidu".to_string(), address: "www.baidu.com".to_string(), command: "".to_string() },
                        HostPreset { id: Uuid::new_v4(), name: "Taobao".to_string(), address: "www.taobao.com".to_string(), command: "".to_string() },
                    ],
                }
            };

            app.manage(AppState {
                monitors: Arc::new(Mutex::new(HashMap::new())),
                settings: Arc::new(Mutex::new(initial_settings)),
                tray_cache: Arc::new(Mutex::new(HashMap::new())),
                is_visible_flag: Arc::new(std::sync::atomic::AtomicBool::new(true)),
                last_click: Arc::new(std::sync::Mutex::new(std::time::Instant::now())),
            });

            // 2. Initialize System Tray (Now safe to use state in callbacks)
            use tauri::menu::{Menu, MenuItem};
            use tauri::tray::TrayIconBuilder;
            
            let quit_i = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>).unwrap();
            let show_i = MenuItem::with_id(app, "show", "Show Ping Monitor", true, None::<&str>).unwrap();
            let menu = Menu::with_items(app, &[&show_i, &quit_i]).unwrap();
            
            let _tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .icon(app.default_window_icon().unwrap().clone())
                .on_menu_event(|app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "show" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                   use tauri::tray::{TrayIconEvent, MouseButton};
                   if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                       let app = tray.app_handle();
                       let state = app.state::<AppState>();
                       
                       // Debounce
                       let mut last_click = state.last_click.lock().unwrap();
                       if last_click.elapsed() < std::time::Duration::from_millis(300) {
                           return;
                       }
                       *last_click = std::time::Instant::now();
                       
                       let is_visible = state.is_visible_flag.load(std::sync::atomic::Ordering::Relaxed);
                       
                       if let Some(window) = app.get_webview_window("main") {
                            if is_visible {
                                let _ = window.hide();
                                state.is_visible_flag.store(false, std::sync::atomic::Ordering::Relaxed);
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                                state.is_visible_flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                       }
                   }
                })
                .build(app)?;

            // 3. Window Event Listener (Now safe because state is managed)
            if let Some(window) = app_handle.get_webview_window("main") {
                let flag = app_handle.state::<AppState>().is_visible_flag.clone();
                window.on_window_event(move |event| {
                    use tauri::WindowEvent;
                    match event {
                        WindowEvent::CloseRequested { .. } | WindowEvent::Destroyed => {
                            flag.store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                        WindowEvent::Focused(focused) => {
                            if *focused {
                                flag.store(true, std::sync::atomic::Ordering::Relaxed);
                            }
                        }
                        _ => {}
                    }
                });
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            start_monitoring, 
            stop_monitoring, 
            add_host, 
            update_host,
            remove_host,
            get_hosts,
            get_settings,
            apply_settings,
            start_all,
            stop_all
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
