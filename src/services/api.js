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
  // 更新用户资料
  async updateProfile(data) {
    let result = null;
    try {
      const token = localStorage.getItem('token'); 

      if (!token) {
        throw new Error('未登录或Token缺失');
      }

      const response = await fetch(`${API_BASE_URL}/accounts/profile/`, {
        method: 'PATCH', 
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Token ${token}`, // 使用Token认证
        },
        body: JSON.stringify(data), // 发送 { bio: 'new bio' }
      });

      // 尝试解析JSON响应
      try {
        result = await response.json();
      } catch (e) {
        // 如果没有JSON响应体，例如204 No Content，如果状态码ok则视为成功
        if (response.ok) {
           result = { message: '更新成功' };
        } else {
           result = null;
        }
      }

      if (!response.ok) {
        // 尝试从API响应的JSON中获取错误信息
        const errorMessage = (result && result.detail) ? result.detail : '更新失败';
        throw new Error(errorMessage);
      }

      return {
        success: true,
        message: '个人简介更新成功',
        data: result
      };
    } catch (error) {
      console.error('更新用户资料错误:', error);
      return {
        success: false,
        message: error instanceof Error ? error.message : '网络错误',
        data: result
      };
    }
  }

}

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

export const apiService = new ApiService();

