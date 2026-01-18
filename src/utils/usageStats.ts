import { invoke } from '@tauri-apps/api/core';
import type { UsageStats } from '../types';

export const DEFAULT_USAGE_STATS: UsageStats = {
  totalRecordingMs: 0,
  totalRecordingCount: 0,
  totalRecognizedChars: 0,
};

/**
 * 从后端加载统计数据
 * 注意：统计数据由后端全权负责，前端只负责显示
 */
export const loadUsageStats = async (): Promise<UsageStats> => {
  try {
    const stats = await invoke<UsageStats>('load_usage_stats');
    console.log('[UsageStats] 从后端加载统计数据:', stats);
    return stats;
  } catch (error) {
    console.error('加载统计数据失败:', error);
    return DEFAULT_USAGE_STATS;
  }
};
