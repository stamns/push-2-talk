// src/OverlayWindow.tsx
// 录音状态悬浮窗组件 - iOS风格的精美设计

import { useState, useEffect, useRef } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";

// 音频级别事件 payload 类型
interface AudioLevelPayload {
  level: number;
}

// 状态类型
type OverlayStatus = "recording" | "transcribing";

// 声波条组件 - 纯白色极简设计
function WaveBar({ height }: { height: number }) {
  return (
    <div
      className="wave-bar"
      style={{ height: `${height}px` }}
    />
  );
}

// 声波动画组件 - 仅显示声波条，无任何文字
function WaveformBars({ level }: { level: number }) {
  // 9个条，创造更密集的声波效果（类似截图）
  const barMultipliers = [0.4, 0.6, 0.8, 0.95, 1.0, 0.95, 0.8, 0.6, 0.4];

  // 最小高度 4px，最大高度 24px
  const minHeight = 4;
  const maxHeight = 24;

  // 放大音量让跳动更明显
  const amplifiedLevel = Math.min(level * 1.5, 1.0);

  return (
    <div className="wave-container">
      {barMultipliers.map((multiplier, i) => {
        const height = minHeight + (amplifiedLevel * multiplier * (maxHeight - minHeight));
        return <WaveBar key={i} height={height} />;
      })}
    </div>
  );
}

// 转写加载组件 - 点阵 + 旋转太阳图标（如截图所示）
function LoadingIndicator() {
  return (
    <div className="loading-container">
      {/* 左侧点阵 */}
      <div className="dots-container">
        {[...Array(9)].map((_, i) => (
          <div key={i} className="dot" style={{ animationDelay: `${i * 0.1}s` }} />
        ))}
      </div>
      {/* 右侧旋转图标 */}
      <div className="spinner-icon">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
          <circle cx="12" cy="12" r="4" />
          <line x1="12" y1="2" x2="12" y2="6" />
          <line x1="12" y1="18" x2="12" y2="22" />
          <line x1="2" y1="12" x2="6" y2="12" />
          <line x1="18" y1="12" x2="22" y2="12" />
          <line x1="4.93" y1="4.93" x2="7.76" y2="7.76" />
          <line x1="16.24" y1="16.24" x2="19.07" y2="19.07" />
          <line x1="4.93" y1="19.07" x2="7.76" y2="16.24" />
          <line x1="16.24" y1="7.76" x2="19.07" y2="4.93" />
        </svg>
      </div>
    </div>
  );
}

// 松手模式控制组件
function LockedControls({
  onFinish,
  onCancel,
  level,
  disabled
}: {
  onFinish: () => void;
  onCancel: () => void;
  level: number;
  disabled: boolean;
}) {
  // 5 条音波，中间最高，两边递减（对称分布）
  const barMultipliers = [0.5, 0.8, 1.0, 0.8, 0.5];

  // 最小高度和最大高度
  const minHeight = 4;
  const maxHeight = 20;

  // 放大音量让跳动更明显
  const amplifiedLevel = Math.min(level * 1.8, 1.0);

  return (
    <div className="locked-controls">
      {/* 取消按钮 */}
      <button
        onClick={onCancel}
        disabled={disabled}
        className={`locked-btn locked-btn-cancel ${disabled ? 'opacity-50' : ''}`}
        title="取消 (Esc)"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
          <line x1="18" y1="6" x2="6" y2="18" />
          <line x1="6" y1="6" x2="18" y2="18" />
        </svg>
      </button>

      {/* 中间 5 条音波 */}
      <div className="locked-wave-mini">
        {barMultipliers.map((multiplier, i) => {
          const height = minHeight + (amplifiedLevel * multiplier * (maxHeight - minHeight));
          return (
            <div
              key={i}
              className="wave-bar-mini"
              style={{ height: `${height}px` }}
            />
          );
        })}
      </div>

      {/* 完成按钮 */}
      <button
        onClick={onFinish}
        disabled={disabled}
        className={`locked-btn locked-btn-finish ${disabled ? 'opacity-50' : ''}`}
        title="发送 (Enter)"
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="3" strokeLinecap="round" strokeLinejoin="round">
          <polyline points="20 6 9 17 4 12" />
        </svg>
      </button>
    </div>
  );
}

// 主悬浮窗组件
export default function OverlayWindow() {
  const [audioLevel, setAudioLevel] = useState(0);
  const [status, setStatus] = useState<OverlayStatus>("recording");
  const [isLocked, setIsLocked] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);
  // 使用 ref 来存储平滑值，避免闭包问题
  const smoothedLevelRef = useRef(0);
  // 标记监听器是否已设置
  const listenersSetup = useRef(false);

  useEffect(() => {
    // 防止重复设置监听器
    if (listenersSetup.current) return;
    listenersSetup.current = true;

    const unlistenFns: UnlistenFn[] = [];

    // 立即设置监听器（不使用 async wrapper）
    const setup = async () => {
      // 监听音频级别更新
      const unlistenAudioLevel = await listen<AudioLevelPayload>("audio_level_update", (event) => {
        const newLevel = event.payload.level;
        // 更激进的平滑处理：快速上升，较快下降，保持动感
        if (newLevel > smoothedLevelRef.current) {
          // 上升时快速响应
          smoothedLevelRef.current = smoothedLevelRef.current * 0.3 + newLevel * 0.7;
        } else {
          // 下降时也保持一定速度，避免粘滞感
          smoothedLevelRef.current = smoothedLevelRef.current * 0.6 + newLevel * 0.4;
        }
        setAudioLevel(smoothedLevelRef.current);
      });
      unlistenFns.push(unlistenAudioLevel);

      // 监听录音开始
      const unlistenStart = await listen("recording_started", () => {
        setStatus("recording");
        setIsLocked(false);
        setIsSubmitting(false);
        smoothedLevelRef.current = 0;
        setAudioLevel(0);
      });
      unlistenFns.push(unlistenStart);

      // 监听录音锁定（松手模式）
      const unlistenLocked = await listen("recording_locked", () => {
        console.log("进入松手模式");
        setIsLocked(true);
        setIsSubmitting(false);
      });
      unlistenFns.push(unlistenLocked);

      // 监听录音停止/转写开始
      const unlistenStop = await listen("recording_stopped", () => {
        setStatus("transcribing");
      });
      unlistenFns.push(unlistenStop);

      const unlistenTranscribing = await listen("transcribing", () => {
        setStatus("transcribing");
      });
      unlistenFns.push(unlistenTranscribing);

      // 监听转写完成
      const unlistenComplete = await listen("transcription_complete", () => {
        setStatus("recording");
        setIsLocked(false);
        setIsSubmitting(false);
        smoothedLevelRef.current = 0;
        setAudioLevel(0);
      });
      unlistenFns.push(unlistenComplete);

      // 监听错误
      const unlistenError = await listen("error", () => {
        setStatus("recording");
        setIsLocked(false);
        setIsSubmitting(false);
        smoothedLevelRef.current = 0;
        setAudioLevel(0);
      });
      unlistenFns.push(unlistenError);

      // 监听取消
      const unlistenCancel = await listen("transcription_cancelled", () => {
        setStatus("recording");
        setIsLocked(false);
        setIsSubmitting(false);
        smoothedLevelRef.current = 0;
        setAudioLevel(0);
      });
      unlistenFns.push(unlistenCancel);
    };

    setup();

    // 清理函数
    return () => {
      unlistenFns.forEach(fn => fn());
      listenersSetup.current = false;
    };
  }, []);

  // 超时保护机制：如果转写状态超过 15 秒，强制调用隐藏
  useEffect(() => {
    if (status === "transcribing") {
      const timeout = setTimeout(async () => {
        console.warn("转写超时 15 秒，强制调用隐藏悬浮窗");
        try {
          await invoke("hide_overlay");
          setStatus("recording");
          setIsLocked(false);
          setIsSubmitting(false);
          smoothedLevelRef.current = 0;
          setAudioLevel(0);
        } catch (e) {
          console.error("强制隐藏悬浮窗失败:", e);
        }
      }, 15000);
      return () => clearTimeout(timeout);
    }
  }, [status]);

  // 松手模式超时保护：60 秒后自动取消
  useEffect(() => {
    if (isLocked && !isSubmitting) {
      const timeout = setTimeout(async () => {
        console.warn("松手模式超时 60 秒，自动取消");
        setIsSubmitting(true);
        try {
          await invoke("cancel_locked_recording");
        } catch (e) {
          console.error("取消锁定录音失败:", e);
        }
      }, 60000);
      return () => clearTimeout(timeout);
    }
  }, [isLocked, isSubmitting]);

  // 完成录音（松手模式）
  const handleFinish = async () => {
    if (isSubmitting) return;
    setIsSubmitting(true);
    try {
      await invoke("finish_locked_recording");
    } catch (e) {
      console.error("完成录音失败:", e);
      setIsSubmitting(false);
    }
  };

  // 取消录音（松手模式）
  const handleCancel = async () => {
    if (isSubmitting) return;
    setIsSubmitting(true);
    try {
      await invoke("cancel_locked_recording");
    } catch (e) {
      console.error("取消录音失败:", e);
      setIsSubmitting(false);
    }
  };

  return (
    <div className="overlay-root">
      <div className={`overlay-pill ${isLocked ? 'overlay-pill-locked' : ''}`}>
        {status === "recording" ? (
          isLocked ? (
            <LockedControls
              onFinish={handleFinish}
              onCancel={handleCancel}
              level={audioLevel}
              disabled={isSubmitting}
            />
          ) : (
            <WaveformBars level={audioLevel} />
          )
        ) : (
          <LoadingIndicator />
        )}
      </div>
    </div>
  );
}
