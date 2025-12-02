const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000/api';

class ApiService {
  // 注册方法
  async register(data) {
    try {
      const response = await fetch(`${API_BASE_URL}/accounts/register/`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
        redirect: 'follow',
      });

      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || result.detail || '注册失败');
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
        error: String(error)
      };
    }
  }
  
  // 登录方法
  async login(data) {
    try {
      const response = await fetch(`${API_BASE_URL}/accounts/login/`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify(data),
      });

      const result = await response.json();
      
      if (!response.ok) {
        throw new Error(result.error || result.detail || '登录失败');
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
        error: String(error)
      };
    }
  }
}

export const apiService = new ApiService();