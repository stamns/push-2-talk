// src-tauri/src/config.rs

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashSet;
use anyhow::Result;

/// 热键配置支持的按键类型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyKey {
    // 修饰键
    ControlLeft,
    ControlRight,
    ShiftLeft,
    ShiftRight,
    AltLeft,
    AltRight,
    MetaLeft,   // Win/Cmd 左
    MetaRight,  // Win/Cmd 右

    // 功能键
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // 常用键
    Space,
    Tab,
    CapsLock,
    Escape,

    // 字母键
    KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ,
    KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT,
    KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ,

    // 数字键
    Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7, Num8, Num9,

    // 方向键
    Up, Down, Left, Right,

    // 编辑键
    Return, Backspace, Delete, Insert, Home, End, PageUp, PageDown,
}

impl HotkeyKey {
    /// 判断是否为修饰键
    pub fn is_modifier(&self) -> bool {
        matches!(self,
            HotkeyKey::ControlLeft | HotkeyKey::ControlRight |
            HotkeyKey::ShiftLeft | HotkeyKey::ShiftRight |
            HotkeyKey::AltLeft | HotkeyKey::AltRight |
            HotkeyKey::MetaLeft | HotkeyKey::MetaRight
        )
    }

    /// 判断是否为功能键
    pub fn is_function_key(&self) -> bool {
        matches!(self,
            HotkeyKey::F1 | HotkeyKey::F2 | HotkeyKey::F3 | HotkeyKey::F4 |
            HotkeyKey::F5 | HotkeyKey::F6 | HotkeyKey::F7 | HotkeyKey::F8 |
            HotkeyKey::F9 | HotkeyKey::F10 | HotkeyKey::F11 | HotkeyKey::F12
        )
    }

    /// 获取显示名称（用于日志和调试）
    pub fn display_name(&self) -> &'static str {
        match self {
            HotkeyKey::ControlLeft => "Ctrl(左)",
            HotkeyKey::ControlRight => "Ctrl(右)",
            HotkeyKey::ShiftLeft => "Shift(左)",
            HotkeyKey::ShiftRight => "Shift(右)",
            HotkeyKey::AltLeft => "Alt(左)",
            HotkeyKey::AltRight => "Alt(右)",
            HotkeyKey::MetaLeft => "Win(左)",
            HotkeyKey::MetaRight => "Win(右)",
            HotkeyKey::Space => "Space",
            HotkeyKey::Tab => "Tab",
            HotkeyKey::CapsLock => "CapsLock",
            HotkeyKey::Escape => "Esc",
            HotkeyKey::F1 => "F1", HotkeyKey::F2 => "F2", HotkeyKey::F3 => "F3",
            HotkeyKey::F4 => "F4", HotkeyKey::F5 => "F5", HotkeyKey::F6 => "F6",
            HotkeyKey::F7 => "F7", HotkeyKey::F8 => "F8", HotkeyKey::F9 => "F9",
            HotkeyKey::F10 => "F10", HotkeyKey::F11 => "F11", HotkeyKey::F12 => "F12",
            HotkeyKey::KeyA => "A", HotkeyKey::KeyB => "B", HotkeyKey::KeyC => "C",
            HotkeyKey::KeyD => "D", HotkeyKey::KeyE => "E", HotkeyKey::KeyF => "F",
            HotkeyKey::KeyG => "G", HotkeyKey::KeyH => "H", HotkeyKey::KeyI => "I",
            HotkeyKey::KeyJ => "J", HotkeyKey::KeyK => "K", HotkeyKey::KeyL => "L",
            HotkeyKey::KeyM => "M", HotkeyKey::KeyN => "N", HotkeyKey::KeyO => "O",
            HotkeyKey::KeyP => "P", HotkeyKey::KeyQ => "Q", HotkeyKey::KeyR => "R",
            HotkeyKey::KeyS => "S", HotkeyKey::KeyT => "T", HotkeyKey::KeyU => "U",
            HotkeyKey::KeyV => "V", HotkeyKey::KeyW => "W", HotkeyKey::KeyX => "X",
            HotkeyKey::KeyY => "Y", HotkeyKey::KeyZ => "Z",
            HotkeyKey::Num0 => "0", HotkeyKey::Num1 => "1", HotkeyKey::Num2 => "2",
            HotkeyKey::Num3 => "3", HotkeyKey::Num4 => "4", HotkeyKey::Num5 => "5",
            HotkeyKey::Num6 => "6", HotkeyKey::Num7 => "7", HotkeyKey::Num8 => "8",
            HotkeyKey::Num9 => "9",
            HotkeyKey::Up => "↑", HotkeyKey::Down => "↓",
            HotkeyKey::Left => "←", HotkeyKey::Right => "→",
            HotkeyKey::Return => "Enter", HotkeyKey::Backspace => "Backspace",
            HotkeyKey::Delete => "Delete", HotkeyKey::Insert => "Insert",
            HotkeyKey::Home => "Home", HotkeyKey::End => "End",
            HotkeyKey::PageUp => "PageUp", HotkeyKey::PageDown => "PageDown",
        }
    }
}

/// 热键配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// 需要同时按下的按键列表
    pub keys: Vec<HotkeyKey>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        // 默认为 Ctrl+Win（向后兼容）
        Self {
            keys: vec![HotkeyKey::ControlLeft, HotkeyKey::MetaLeft],
        }
    }
}

impl HotkeyConfig {
    /// 检查是否包含至少一个修饰键
    pub fn has_modifier(&self) -> bool {
        self.keys.iter().any(|k| k.is_modifier())
    }

    /// 验证热键配置是否有效
    pub fn validate(&self) -> Result<()> {
        if self.keys.is_empty() {
            anyhow::bail!("热键配置不能为空");
        }

        // 允许功能键单独使用，其他按键必须配合修饰键
        let has_function_key = self.keys.iter().any(|k| k.is_function_key());
        if !self.has_modifier() && !has_function_key {
            anyhow::bail!("热键必须包含至少一个修饰键 (Ctrl/Alt/Shift/Win) 或使用功能键 (F1-F12)");
        }

        if self.keys.len() > 4 {
            anyhow::bail!("热键最多支持4个按键组合");
        }

        // 检查是否有重复按键
        let unique_keys: HashSet<_> = self.keys.iter().collect();
        if unique_keys.len() != self.keys.len() {
            anyhow::bail!("热键配置中存在重复的按键");
        }

        Ok(())
    }

    /// 格式化为显示字符串（用于日志）
    pub fn format_display(&self) -> String {
        self.keys.iter()
            .map(|k| k.display_name())
            .collect::<Vec<_>>()
            .join("+")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AsrProvider {
    Qwen,
    Doubao,
    #[serde(rename = "siliconflow")]
    SiliconFlow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrProviderConfig {
    pub provider: AsrProvider,
    pub api_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub access_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AsrConfig {
    pub primary: AsrProviderConfig,
    #[serde(default)]
    pub fallback: Option<AsrProviderConfig>,
    #[serde(default)]
    pub enable_fallback: bool,
}

impl Default for AsrConfig {
    fn default() -> Self {
        Self {
            primary: AsrProviderConfig {
                provider: AsrProvider::Qwen,
                api_key: String::new(),
                app_id: None,
                access_token: None,
            },
            fallback: None,
            enable_fallback: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub dashscope_api_key: String,
    #[serde(default)]
    pub siliconflow_api_key: String,
    #[serde(default)]
    pub asr_config: AsrConfig,
    #[serde(default = "default_use_realtime_asr")]
    pub use_realtime_asr: bool,
    #[serde(default)]
    pub enable_llm_post_process: bool,
    #[serde(default)]
    pub llm_config: LlmConfig,
    /// 关闭行为: "close" = 直接关闭, "minimize" = 最小化到托盘, None = 每次询问
    #[serde(default)]
    pub close_action: Option<String>,
    /// 热键配置（默认 Ctrl+Win）
    #[serde(default)]
    pub hotkey_config: HotkeyConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmPreset {
    pub id: String,
    pub name: String,
    pub system_prompt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmConfig {
    #[serde(default = "default_llm_endpoint")]
    pub endpoint: String,
    #[serde(default = "default_llm_model")]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
    
    // 新增：预设列表和当前选中的预设ID
    #[serde(default = "default_presets")]
    pub presets: Vec<LlmPreset>,
    #[serde(default = "default_active_preset_id")]
    pub active_preset_id: String,
}

fn default_llm_endpoint() -> String {
    "https://open.bigmodel.cn/api/paas/v4/chat/completions".to_string()
}

fn default_llm_model() -> String {
    "glm-4-flash-250414".to_string()
}

// 默认预设生成逻辑
fn default_presets() -> Vec<LlmPreset> {
    vec![
        LlmPreset {
            id: "polishing".to_string(),
            name: "文本润色".to_string(),
            system_prompt: "你是一个语音转写润色助手。请在不改变原意的前提下：1）删除重复或意义相近的句子；2）合并同一主题的内容；3）去除「嗯」「啊」等口头禅；4）保留数字与关键信息；5）相关数字和时间不要使用中文；6）整理成自然的段落。输出纯文本即可。".to_string(),
        },
        LlmPreset {
            id: "translation".to_string(),
            name: "中译英".to_string(),
            system_prompt: "你是一个专业的翻译助手。请将用户的中文语音转写内容翻译成地道、流畅的英文。不要输出任何解释性文字，只输出翻译结果。".to_string(),
        }
    ]
}

fn default_active_preset_id() -> String {
    "polishing".to_string()
}

// 为了兼容旧版本配置，如果反序列化时 presets 为空，手动填充默认值
impl Default for LlmConfig {
    fn default() -> Self {
        Self {
            endpoint: default_llm_endpoint(),
            model: default_llm_model(),
            api_key: String::new(),
            presets: default_presets(),
            active_preset_id: default_active_preset_id(),
        }
    }
}

fn default_use_realtime_asr() -> bool {
    true
}

impl AppConfig {
    pub fn new() -> Self {
        Self {
            dashscope_api_key: String::new(),
            siliconflow_api_key: String::new(),
            asr_config: AsrConfig::default(),
            use_realtime_asr: default_use_realtime_asr(),
            enable_llm_post_process: false,
            llm_config: LlmConfig::default(),
            close_action: None,
            hotkey_config: HotkeyConfig::default(),
        }
    }

    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("无法获取配置目录"))?;
        let app_dir = config_dir.join("PushToTalk");
        std::fs::create_dir_all(&app_dir)?;
        Ok(app_dir.join("config.json"))
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        tracing::info!("尝试从以下路径加载配置: {:?}", path);
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let mut config: AppConfig = serde_json::from_str(&content)?;

            // 迁移逻辑：如果 asr_config 为空但旧字段有值，自动迁移
            if config.asr_config.primary.api_key.is_empty() && !config.dashscope_api_key.is_empty() {
                tracing::info!("检测到旧配置格式，自动迁移到新格式");
                config.asr_config.primary = AsrProviderConfig {
                    provider: AsrProvider::Qwen,
                    api_key: config.dashscope_api_key.clone(),
                    app_id: None,
                    access_token: None,
                };
                if !config.siliconflow_api_key.is_empty() {
                    config.asr_config.fallback = Some(AsrProviderConfig {
                        provider: AsrProvider::SiliconFlow,
                        api_key: config.siliconflow_api_key.clone(),
                        app_id: None,
                        access_token: None,
                    });
                    config.asr_config.enable_fallback = true;
                }
            }

            if config.llm_config.presets.is_empty() {
                 tracing::info!("检测到预设列表为空，用户可能删除了所有预设");
            }

            tracing::info!("配置加载成功");
            Ok(config)
        } else {
            tracing::warn!("配置文件不存在，创建并返回默认配置");
            Ok(Self::new())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        let content = serde_json::to_string_pretty(self)?;
        tracing::info!("保存配置到: {:?}", path);
        std::fs::write(&path, content)?;
        tracing::info!("配置保存成功");
        Ok(())
    }
}