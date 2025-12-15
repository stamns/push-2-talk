// 全局快捷键监听模块
use rdev::{listen, Event, EventType, Key};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use anyhow::Result;

// 看门狗检查间隔（毫秒）- 每隔此时间检查一次按键状态
const WATCHDOG_INTERVAL_MS: u64 = 100;
// 按键释放后的稳定时间（毫秒）- 检测到按键释放后，等待此时间确认状态稳定
const KEY_RELEASE_STABLE_MS: u64 = 200;

/// 热键状态，合并到单一结构体避免竞态条件
#[derive(Debug, Clone)]
struct HotkeyState {
    is_recording: bool,
    ctrl_pressed: bool,
    win_pressed: bool,
    watchdog_running: bool,
}

impl Default for HotkeyState {
    fn default() -> Self {
        Self {
            is_recording: false,
            ctrl_pressed: false,
            win_pressed: false,
            watchdog_running: false,
        }
    }
}

pub struct HotkeyService {
    state: Arc<Mutex<HotkeyState>>,
}

impl HotkeyService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(HotkeyState::default())),
        }
    }

    pub fn start<F1, F2>(&self, on_start: F1, on_stop: F2) -> Result<()>
    where
        F1: Fn() + Send + 'static,
        F2: Fn() + Send + Sync + 'static,
    {
        tracing::info!("启动快捷键监听服务 (Ctrl+Win)");

        let state = Arc::clone(&self.state);

        // 将 on_stop 包装成 Arc 以便在多个线程中调用（主回调和看门狗线程）
        let on_stop_arc = Arc::new(on_stop);

        thread::spawn(move || {
            tracing::info!("快捷键监听线程已启动");
            let mut first_key_logged = false;

            let callback = move |event: Event| {
                // 第一次检测到按键时记录（用于确认 rdev 工作正常）
                if !first_key_logged && matches!(event.event_type, EventType::KeyPress(_)) {
                    first_key_logged = true;
                    tracing::info!("✓ rdev 正常工作 - 已检测到键盘事件");
                }

                match event.event_type {
                    EventType::KeyPress(key) => {
                        let is_ctrl_key = matches!(key, Key::ControlLeft | Key::ControlRight);
                        let is_win_key = matches!(key, Key::MetaLeft | Key::MetaRight);

                        if !is_ctrl_key && !is_win_key {
                            return; // 忽略其他按键
                        }

                        // 使用单一锁操作所有状态
                        let mut s = state.lock().unwrap();

                        if is_ctrl_key {
                            tracing::debug!("检测到 Ctrl 键按下");
                            s.ctrl_pressed = true;
                        }
                        if is_win_key {
                            tracing::debug!("检测到 Win 键按下");
                            s.win_pressed = true;
                        }

                        // 检查是否按下了 Ctrl+Win 且未在录音
                        if s.ctrl_pressed && s.win_pressed && !s.is_recording {
                            s.is_recording = true;
                            tracing::info!("检测到快捷键按下: Ctrl+Win，开始录音");

                            // 检查是否已有看门狗在运行，防止重复启动
                            if s.watchdog_running {
                                tracing::debug!("看门狗已在运行，跳过启动");
                                drop(s);
                                on_start();
                                return;
                            }

                            s.watchdog_running = true;
                            drop(s); // 释放锁后再启动看门狗线程

                            // 启动看门狗线程
                            let state_wd = Arc::clone(&state);
                            let on_stop_wd = Arc::clone(&on_stop_arc);

                            thread::spawn(move || {
                                tracing::debug!("看门狗线程已启动");
                                let mut release_detected_count: u64 = 0;
                                let required_count = (KEY_RELEASE_STABLE_MS / WATCHDOG_INTERVAL_MS).max(1);

                                loop {
                                    thread::sleep(Duration::from_millis(WATCHDOG_INTERVAL_MS));

                                    // 使用单一锁检查所有状态
                                    let s = state_wd.lock().unwrap();

                                    // 检查看门狗是否应该停止
                                    if !s.watchdog_running {
                                        tracing::debug!("看门狗线程正常退出（标志已清除）");
                                        break;
                                    }

                                    // 检查录音状态
                                    if !s.is_recording {
                                        tracing::debug!("看门狗线程正常退出（录音已停止）");
                                        break;
                                    }

                                    // 检查按键状态：只要有一个键释放了，就应该停止
                                    let should_stop = !s.ctrl_pressed || !s.win_pressed;
                                    drop(s); // 释放锁

                                    if should_stop {
                                        release_detected_count += 1;
                                        tracing::debug!(
                                            "看门狗检测到按键释放，计数: {}/{}",
                                            release_detected_count,
                                            required_count
                                        );

                                        // 需要连续检测到释放状态才触发（防止误判）
                                        if release_detected_count >= required_count {
                                            // 重新获取锁进行状态修改
                                            let mut s = state_wd.lock().unwrap();
                                            if s.is_recording {
                                                s.is_recording = false;
                                                s.watchdog_running = false;
                                                drop(s); // 先释放锁再调用回调

                                                tracing::warn!(
                                                    "看门狗检测到按键释放事件丢失，强制停止录音"
                                                );
                                                on_stop_wd();
                                            }
                                            break;
                                        }
                                    } else {
                                        // 两个键都还按着，重置计数
                                        release_detected_count = 0;
                                    }
                                }

                                // 确保退出时清理看门狗标志
                                let mut s = state_wd.lock().unwrap();
                                s.watchdog_running = false;
                            });

                            on_start();
                        }
                    }
                    EventType::KeyRelease(key) => {
                        let is_ctrl_key = matches!(key, Key::ControlLeft | Key::ControlRight);
                        let is_win_key = matches!(key, Key::MetaLeft | Key::MetaRight);

                        if !is_ctrl_key && !is_win_key {
                            return; // 忽略其他按键
                        }

                        // 使用单一锁操作所有状态
                        let mut s = state.lock().unwrap();

                        // 更新按键状态
                        if is_ctrl_key {
                            tracing::debug!("检测到 Ctrl 键释放");
                            s.ctrl_pressed = false;
                        }
                        if is_win_key {
                            tracing::debug!("检测到 Win 键释放");
                            s.win_pressed = false;
                        }

                        // 没有在录音，忽略
                        if !s.is_recording {
                            return;
                        }

                        // 只要有一个键释放了，就停止录音
                        s.is_recording = false;
                        s.watchdog_running = false;
                        drop(s); // 释放锁后再调用回调

                        tracing::info!("检测到快捷键释放，停止录音");
                        let on_stop_ref = Arc::clone(&on_stop_arc);
                        on_stop_ref();
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
}
