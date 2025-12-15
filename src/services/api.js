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

  async changePassword(data, refreshToken) { 
    let result = null;

    try {
      const authToken = localStorage.getItem('token'); 
      if (!authToken) {
        throw new Error('未登录或主认证Token缺失'); 
      }
      if (!refreshToken) {
        throw new Error('Refresh Token缺失'); 
      }
      
      // 构造请求体数据
      const requestData = {
        old_password: data.old_password,
        new_password: data.new_password,
        new_password2: data.new_password2, 
        refresh_token: refreshToken, 
      };

      const response = await fetch(`${API_BASE_URL}/accounts/change-password/`,  {
        method: 'POST', 
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Token ${authToken}`,
        },
        body: JSON.stringify(requestData), 
      });

      // 尝试解析JSON响应
      try {
        result = await response.json();
      } catch (e) {
        if (response.ok) {
           result = { message: '密码更新成功' };
        } else {
           result = null;
        }
      }

      if (!response.ok) {
        const errorDetails = result || {};
        const getErrorMessages = (errors) => errors && Array.isArray(errors) ? errors.join(' ') : '';
        
        // 提取错误信息
        const errorMessage = getErrorMessages(errorDetails.new_password) ||
                             getErrorMessages(errorDetails.new_password2) ||
                             getErrorMessages(errorDetails.old_password) ||
                             getErrorMessages(errorDetails.non_field_errors) ||
                             errorDetails.detail ||
                             '密码修改失败，请检查输入';
                             
        throw new Error(errorMessage);
      }

      return {
        success: true,
        message: '密码修改成功',
        data: result
      };
    } catch (error) {
      console.error('修改密码错误:', error);
      return {
        success: false,
        message: error instanceof Error ? error.message : '网络错误',
        data: result
      };
    }
  }

  /**
   * 删除当前用户账户。
   * @returns {Promise<{success: boolean, message: string, data: object|null}>}
   */
  async deleteAccount() {
    let result = null;
    try {
      const authToken = localStorage.getItem('token'); 

      if (!authToken) {
        // 统一鉴权失败的返回方式，通过 throw new Error 最终被 catch 捕获
        throw new Error('未登录或Token缺失');
      }

      const response = await fetch(`${API_BASE_URL}/accounts/delete/`, {
        method: 'DELETE', // 使用 DELETE 方法
        headers: {
          'Authorization': `Token ${authToken}`, // 使用Token认证
        },
      });

      // 删除成功通常返回 204 No Content，没有 body
      if (response.status === 204) {
        return {
          success: true,
          message: '账户已删除',
          data: null
        };
      }
      
      // 尝试解析JSON响应以获取可能的错误信息
      try {
        result = await response.json();
      } catch (e) {
        // 忽略没有 JSON body 的情况
      }

      if (!response.ok) {
        // 尝试从API响应的JSON中获取错误信息
        const errorMessage = (result && result.detail) ? result.detail : '账户删除失败，请稍后重试';
        throw new Error(errorMessage);
      }
      
      // 如果返回 200/201 且 response.ok 为 true (极少见于 DELETE)，我们仍然认为成功
      return {
        success: true,
        message: '账户已删除',
        data: result
      };

    } catch (error) {
      console.error('删除账户错误:', error);
      return {
        success: false,
        message: error instanceof Error ? error.message : '网络错误',
        data: result
      };
    }
  }

  /**
   * 上传当前用户配置到后端。
   * @param {string} configContent - 配置文件的内容字符串 (例如 JSON 字符串)。
   * @returns {Promise<{success: boolean, message: string, data: object|null}>}
   */
  async uploadConfig(configContent) {
    let result = null;
    try {
      const token = localStorage.getItem('token'); 

      if (!token) {
        throw new Error('未登录或Token缺失');
      }

      const formData = new FormData();
      // 创建一个 Blob 来模拟文件。后端 Python 代码中期望的文件字段名为 'file'。
      // 'config.json' 是提供给后端的文件名。
      const configFileBlob = new Blob([configContent], { type: 'application/json' });
      formData.append('file', configFileBlob, 'config.json'); 

      const response = await fetch(`${API_BASE_URL}/sync/config/`, {
        method: 'POST',
        headers: {
          // fetch 在使用 FormData 时会自动设置 Content-Type: multipart/form-data
          'Authorization': `Token ${token}`, // 使用Token认证
        },
        body: formData, // 发送 FormData
      });

      // 尝试解析JSON响应
      try {
        result = await response.json();
      } catch (e) {
        // 如果没有JSON响应体，例如204 No Content，如果状态码ok则视为成功
        if (response.ok) {
           result = { message: '配置上传成功' };
        } else {
           result = null;
        }
      }

      if (!response.ok) {
        // 尝试从API响应的JSON中获取错误信息
        const errorMessage = (result && result.detail) ? result.detail : '配置上传失败';
        throw new Error(errorMessage);
      }

      return {
        success: true,
        message: '配置上传成功',
        data: result
      };
    } catch (error) {
      console.error('上传配置错误:', error);
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

