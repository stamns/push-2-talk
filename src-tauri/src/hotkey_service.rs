// 全局快捷键监听模块 - 单例模式重构
use rdev::{listen, Event, EventType, Key};
use std::sync::{Arc, Mutex, RwLock};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;
use std::collections::HashSet;
use anyhow::Result;
use crate::config::{HotkeyConfig, HotkeyKey};

// 看门狗检查间隔（毫秒）
const WATCHDOG_INTERVAL_MS: u64 = 100;
// 按键释放后的稳定时间（毫秒）
const KEY_RELEASE_STABLE_MS: u64 = 200;

/// 热键状态
#[derive(Debug, Default)]
struct HotkeyState {
    is_recording: bool,
    pressed_keys: HashSet<HotkeyKey>,
    watchdog_running: bool,
}

/// 回调函数类型
type Callback = Arc<dyn Fn() + Send + Sync>;

/// 单例热键服务
pub struct HotkeyService {
    /// 服务是否激活（控制是否响应热键事件）
    is_active: Arc<AtomicBool>,
    /// 当前热键配置（可动态更新）
    config: Arc<RwLock<HotkeyConfig>>,
    /// 内部状态
    state: Arc<Mutex<HotkeyState>>,
    /// 监听线程是否已启动
    listener_started: Arc<AtomicBool>,
    /// 回调函数
    on_start: Arc<RwLock<Option<Callback>>>,
    on_stop: Arc<RwLock<Option<Callback>>>,
}

impl HotkeyService {
    pub fn new() -> Self {
        Self {
            is_active: Arc::new(AtomicBool::new(false)),
            config: Arc::new(RwLock::new(HotkeyConfig::default())),
            state: Arc::new(Mutex::new(HotkeyState::default())),
            listener_started: Arc::new(AtomicBool::new(false)),
            on_start: Arc::new(RwLock::new(None)),
            on_stop: Arc::new(RwLock::new(None)),
        }
    }

    /// 将 rdev::Key 转换为 HotkeyKey
    fn rdev_to_hotkey_key(key: Key) -> Option<HotkeyKey> {
        match key {
            Key::ControlLeft => Some(HotkeyKey::ControlLeft),
            Key::ControlRight => Some(HotkeyKey::ControlRight),
            Key::ShiftLeft => Some(HotkeyKey::ShiftLeft),
            Key::ShiftRight => Some(HotkeyKey::ShiftRight),
            Key::Alt => Some(HotkeyKey::AltLeft),
            Key::AltGr => Some(HotkeyKey::AltRight),
            Key::MetaLeft => Some(HotkeyKey::MetaLeft),
            Key::MetaRight => Some(HotkeyKey::MetaRight),
            Key::Space => Some(HotkeyKey::Space),
            Key::Tab => Some(HotkeyKey::Tab),
            Key::CapsLock => Some(HotkeyKey::CapsLock),
            Key::Escape => Some(HotkeyKey::Escape),
            Key::F1 => Some(HotkeyKey::F1),
            Key::F2 => Some(HotkeyKey::F2),
            Key::F3 => Some(HotkeyKey::F3),
            Key::F4 => Some(HotkeyKey::F4),
            Key::F5 => Some(HotkeyKey::F5),
            Key::F6 => Some(HotkeyKey::F6),
            Key::F7 => Some(HotkeyKey::F7),
            Key::F8 => Some(HotkeyKey::F8),
            Key::F9 => Some(HotkeyKey::F9),
            Key::F10 => Some(HotkeyKey::F10),
            Key::F11 => Some(HotkeyKey::F11),
            Key::F12 => Some(HotkeyKey::F12),
            Key::KeyA => Some(HotkeyKey::KeyA),
            Key::KeyB => Some(HotkeyKey::KeyB),
            Key::KeyC => Some(HotkeyKey::KeyC),
            Key::KeyD => Some(HotkeyKey::KeyD),
            Key::KeyE => Some(HotkeyKey::KeyE),
            Key::KeyF => Some(HotkeyKey::KeyF),
            Key::KeyG => Some(HotkeyKey::KeyG),
            Key::KeyH => Some(HotkeyKey::KeyH),
            Key::KeyI => Some(HotkeyKey::KeyI),
            Key::KeyJ => Some(HotkeyKey::KeyJ),
            Key::KeyK => Some(HotkeyKey::KeyK),
            Key::KeyL => Some(HotkeyKey::KeyL),
            Key::KeyM => Some(HotkeyKey::KeyM),
            Key::KeyN => Some(HotkeyKey::KeyN),
            Key::KeyO => Some(HotkeyKey::KeyO),
            Key::KeyP => Some(HotkeyKey::KeyP),
            Key::KeyQ => Some(HotkeyKey::KeyQ),
            Key::KeyR => Some(HotkeyKey::KeyR),
            Key::KeyS => Some(HotkeyKey::KeyS),
            Key::KeyT => Some(HotkeyKey::KeyT),
            Key::KeyU => Some(HotkeyKey::KeyU),
            Key::KeyV => Some(HotkeyKey::KeyV),
            Key::KeyW => Some(HotkeyKey::KeyW),
            Key::KeyX => Some(HotkeyKey::KeyX),
            Key::KeyY => Some(HotkeyKey::KeyY),
            Key::KeyZ => Some(HotkeyKey::KeyZ),
            Key::Num0 => Some(HotkeyKey::Num0),
            Key::Num1 => Some(HotkeyKey::Num1),
            Key::Num2 => Some(HotkeyKey::Num2),
            Key::Num3 => Some(HotkeyKey::Num3),
            Key::Num4 => Some(HotkeyKey::Num4),
            Key::Num5 => Some(HotkeyKey::Num5),
            Key::Num6 => Some(HotkeyKey::Num6),
            Key::Num7 => Some(HotkeyKey::Num7),
            Key::Num8 => Some(HotkeyKey::Num8),
            Key::Num9 => Some(HotkeyKey::Num9),
            Key::UpArrow => Some(HotkeyKey::Up),
            Key::DownArrow => Some(HotkeyKey::Down),
            Key::LeftArrow => Some(HotkeyKey::Left),
            Key::RightArrow => Some(HotkeyKey::Right),
            Key::Return => Some(HotkeyKey::Return),
            Key::Backspace => Some(HotkeyKey::Backspace),
            Key::Delete => Some(HotkeyKey::Delete),
            Key::Insert => Some(HotkeyKey::Insert),
            Key::Home => Some(HotkeyKey::Home),
            Key::End => Some(HotkeyKey::End),
            Key::PageUp => Some(HotkeyKey::PageUp),
            Key::PageDown => Some(HotkeyKey::PageDown),
            _ => None,
        }
    }

    /// 初始化监听线程（只调用一次）
    pub fn init_listener(&self) -> Result<()> {
        // 防止重复启动
        if self.listener_started.swap(true, Ordering::SeqCst) {
            tracing::debug!("监听线程已启动，跳过重复初始化");
            return Ok(());
        }

        tracing::info!("初始化全局快捷键监听线程");

        let is_active = Arc::clone(&self.is_active);
        let config = Arc::clone(&self.config);
        let state = Arc::clone(&self.state);
        let on_start = Arc::clone(&self.on_start);
        let on_stop = Arc::clone(&self.on_stop);

        thread::spawn(move || {
            tracing::info!("快捷键监听线程已启动");
            let mut first_key_logged = false;

            let callback = move |event: Event| {
                // 检查服务是否激活
                if !is_active.load(Ordering::Relaxed) {
                    return;
                }

                // 第一次检测到按键时记录
                if !first_key_logged && matches!(event.event_type, EventType::KeyPress(_)) {
                    first_key_logged = true;
                    tracing::info!("✓ rdev 正常工作 - 已检测到键盘事件");
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        if let Some(hotkey_key) = Self::rdev_to_hotkey_key(key) {
                            let current_config = config.read().unwrap().clone();
                            let mut s = state.lock().unwrap();

                            s.pressed_keys.insert(hotkey_key);

                            // 严格匹配：检查包含关系 + 数量一致（防止 Ctrl+Space 被 Ctrl+Shift+Space 误触发）
                            let contains_all = current_config.keys.iter().all(|k| s.pressed_keys.contains(k));
                            let count_match = s.pressed_keys.len() == current_config.keys.len();
                            if contains_all && count_match && !s.is_recording {
                                s.is_recording = true;
                                tracing::info!("检测到快捷键按下: {}，开始录音", current_config.format_display());

                                // 启动看门狗
                                if s.watchdog_running {
                                    drop(s);
                                    if let Some(cb) = on_start.read().unwrap().as_ref() {
                                        cb();
                                    }
                                    return;
                                }

                                s.watchdog_running = true;
                                drop(s);

                                // 启动看门狗线程
                                let state_wd = Arc::clone(&state);
                                let config_wd = Arc::clone(&config);
                                let is_active_wd = Arc::clone(&is_active);
                                let on_stop_wd = Arc::clone(&on_stop);

                                thread::spawn(move || {
                                    tracing::debug!("看门狗线程已启动");
                                    let mut release_detected_count: u64 = 0;
                                    let required_count = (KEY_RELEASE_STABLE_MS / WATCHDOG_INTERVAL_MS).max(1);

                                    loop {
                                        thread::sleep(Duration::from_millis(WATCHDOG_INTERVAL_MS));

                                        // 检查服务是否仍然激活
                                        if !is_active_wd.load(Ordering::Relaxed) {
                                            let mut s = state_wd.lock().unwrap();
                                            s.watchdog_running = false;
                                            s.is_recording = false;
                                            tracing::debug!("看门狗线程退出（服务已停止）");
                                            break;
                                        }

                                        let s = state_wd.lock().unwrap();
                                        if !s.watchdog_running || !s.is_recording {
                                            tracing::debug!("看门狗线程正常退出");
                                            break;
                                        }

                                        let current_config = config_wd.read().unwrap();
                                        let all_pressed = current_config.keys.iter().all(|k| s.pressed_keys.contains(k));
                                        drop(current_config);
                                        drop(s);

                                        if !all_pressed {
                                            release_detected_count += 1;
                                            if release_detected_count >= required_count {
                                                let mut s = state_wd.lock().unwrap();
                                                if s.is_recording {
                                                    s.is_recording = false;
                                                    s.watchdog_running = false;
                                                    drop(s);
                                                    tracing::warn!("看门狗检测到按键释放事件丢失，强制停止录音");
                                                    if let Some(cb) = on_stop_wd.read().unwrap().as_ref() {
                                                        cb();
                                                    }
                                                }
                                                break;
                                            }
                                        } else {
                                            release_detected_count = 0;
                                        }
                                    }

                                    let mut s = state_wd.lock().unwrap();
                                    s.watchdog_running = false;
                                });

                                if let Some(cb) = on_start.read().unwrap().as_ref() {
                                    cb();
                                }
                            }
                        }
                    }
                    EventType::KeyRelease(key) => {
                        if let Some(hotkey_key) = Self::rdev_to_hotkey_key(key) {
                            let current_config = config.read().unwrap().clone();
                            let mut s = state.lock().unwrap();

                            s.pressed_keys.remove(&hotkey_key);

                            // 防呆逻辑：如果释放的是修饰键且未录音，检查是否所有修饰键都已释放
                            if hotkey_key.is_modifier() && !s.is_recording {
                                let has_any_modifier = s.pressed_keys.iter().any(|k| k.is_modifier());
                                if !has_any_modifier {
                                    s.pressed_keys.clear();
                                    tracing::debug!("所有修饰键已释放，强制清理按键状态");
                                }
                            }

                            if !s.is_recording {
                                return;
                            }

                            let all_pressed = current_config.keys.iter().all(|k| s.pressed_keys.contains(k));
                            if !all_pressed {
                                s.is_recording = false;
                                s.watchdog_running = false;
                                drop(s);

                                tracing::info!("检测到快捷键释放，停止录音");
                                if let Some(cb) = on_stop.read().unwrap().as_ref() {
                                    cb();
                                }
                            }
                        }
                    }
                    _ => {}
                }
            };

            if let Err(error) = listen(callback) {
                tracing::error!("无法监听键盘事件: {:?}", error);
            }
        });

        Ok(())
    }

    /// 更新配置并激活服务
    pub fn activate<F1, F2>(&self, config: HotkeyConfig, on_start: F1, on_stop: F2) -> Result<()>
    where
        F1: Fn() + Send + Sync + 'static,
        F2: Fn() + Send + Sync + 'static,
    {
        let hotkey_display = config.format_display();
        tracing::info!("激活快捷键服务 ({})", hotkey_display);

        // 更新配置
        *self.config.write().unwrap() = config;

        // 更新回调
        *self.on_start.write().unwrap() = Some(Arc::new(on_start));
        *self.on_stop.write().unwrap() = Some(Arc::new(on_stop));

        // 重置状态
        {
            let mut s = self.state.lock().unwrap();
            s.is_recording = false;
            s.pressed_keys.clear();
            s.watchdog_running = false;
        }

        // 确保监听线程已启动
        self.init_listener()?;

        // 激活服务
        self.is_active.store(true, Ordering::SeqCst);

        Ok(())
    }

    /// 停用服务（不终止线程）
    pub fn deactivate(&self) {
        tracing::info!("停用快捷键服务");
        self.is_active.store(false, Ordering::SeqCst);

        // 重置状态
        let mut s = self.state.lock().unwrap();
        s.is_recording = false;
        s.pressed_keys.clear();
        s.watchdog_running = false;
    }

    /// 检查服务是否激活
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Relaxed)
    }
}
