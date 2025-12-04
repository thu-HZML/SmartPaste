import { invoke } from '@tauri-apps/api/core';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000/api';

class ApiService {
  // 注册方法
  async register(data) {
    let result = null;
    try {
      const response = await fetch(`${API_BASE_URL}/accounts/register/`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
        redirect: 'follow',
      });

      result = await response.json();

      if (!response.ok) {
        throw new Error('注册失败');
      }

      return {
        success: true,
        message: '注册成功',
        data: result
      };
    } catch (error) {
      return {
        success: false,
        message: error instanceof Error ? error.message : '网络错误',
        data: result
      };
    }
  }
  
  // 登录方法
  async login(data) {
    let result = null;
    try {
      const response = await fetch(`${API_BASE_URL}/accounts/login/`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      result = await response.json();
      
      if (!response.ok) {
        throw new Error('登录失败');
      }

      return {
        success: true,
        message: '登录成功',
        data: result
      };
    } catch (error) {
      return {
        success: false,
        message: error instanceof Error ? error.message : '网络错误',
        data: result
      };
    }
  }
}

export const apiService = new ApiService();

/**
 * 清空所有剪贴板历史，包括收藏的内容 (调用 Rust 的 delete_all_data)。
 * @returns {Promise<number>} 受影响的行数
 */
export async function deleteAllData() {
    try {
        const rowsAffected = await invoke('delete_all_data');
        return rowsAffected;
    } catch (error) {
        console.error('Failed to delete all data:', error);
        throw error;
    }
}

/**
 * 清空剪贴板历史，保留已收藏的内容 (调用 Rust 的 delete_unfavorited_data)。
 * @returns {Promise<number>} 受影响的行数
 */
export async function deleteUnfavoritedData() {
    try {
        const rowsAffected = await invoke('delete_unfavorited_data');
        return rowsAffected;
    } catch (error) {
        console.error('Failed to delete unfavorited data:', error);
        throw error;
    }
}