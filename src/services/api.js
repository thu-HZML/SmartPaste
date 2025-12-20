import { invoke } from '@tauri-apps/api/core';

//const API_BASE_URL = 'http://101.42.152.3/api';
 const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8000/api';


// 提取媒体文件的基础 URL 
// 它假设 media 文件路径是相对于 API_BASE_URL 的前缀而言的
const getMediaBaseUrl = () => {
    // 移除 '/api' 或其他 API 路径后缀，以获得服务器根地址
    // 假设 API_BASE_URL 是 http://domain.com/api，则 MediaBaseUrl 是 http://domain.com
    const url = API_BASE_URL.replace(/\/api\/?$/, ''); 
    return url;
}

/**
 * 确保头像 URL 是一个完整的绝对 URL
 * @param {string} avatarPath - 头像路径，可能是相对路径 (/media/...) 或绝对路径 (http://...)
 * @returns {string} 完整的绝对 URL
 */
export const ensureAbsoluteAvatarUrl = (avatarPath) => {
    if (!avatarPath) {
        return '';
    }
    // 如果已经是绝对 URL (http:// 或 https://)，直接返回
    if (avatarPath.startsWith('http')) {
        return avatarPath;
    }
    
    const mediaBaseUrl = getMediaBaseUrl();
    // 确保路径以 / 开始，且 base URL 不以 / 结束
    const cleanPath = avatarPath.startsWith('/') ? avatarPath : `/${avatarPath}`;
    
    return `${mediaBaseUrl}${cleanPath}`;
}

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
      
      const token = result.jwt.access;
      localStorage.setItem('token', token);
      console.log('登录接口返回的完整数据结构:', token);
      
      // 在同步前和返回前，修正头像 URL
      if (result && result.user && result.user.avatar) {
          result.user.avatar = ensureAbsoluteAvatarUrl(result.user.avatar);
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
          'Authorization': `Bearer ${token}`, // 使用Token认证
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

  // 修改密码
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
          'Authorization': `Bearer ${authToken}`,
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
          'Authorization': `Bearer ${authToken}`, // 使用Token认证
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
   * 接口: POST /api/accounts/avatar/
   * @param {File | Blob} fileObject - 要上传的 File 或 Blob 对象。
   * @returns {Promise<{success: boolean, message: string, data: object|null}>} 
   */
  async uploadAvatar(fileObject) {
    let result = null;
    try {
      const token = localStorage.getItem('token'); 

      if (!token) {
        throw new Error('未登录或Token缺失');
      }

      const formData = new FormData();
      // 后端期望的文件字段名为 'avatar'。
      // fileObject.name 是文件名，如果缺失，使用默认名。
      formData.append('avatar', fileObject, fileObject.name || 'avatar_upload'); 

      const response = await fetch(`${API_BASE_URL}/accounts/avatar/`, {
        method: 'POST',
        headers: {
          'Authorization': `Bearer ${token}`,
          // fetch 在使用 FormData 时会自动设置 Content-Type: multipart/form-data
        },
        body: formData,
      });

      try {
        result = await response.json();
      } catch (e) {
        if (response.ok) {
           result = { message: '头像上传成功' };
        } else {
           result = null;
        }
      }

      if (!response.ok) {
        // 尝试解析错误信息，优先 avatar 字段的错误
        let errorMessage = '头像上传失败';
        if (result) {
          if (result.avatar && Array.isArray(result.avatar)) {
            errorMessage = result.avatar.join(' ');
          } else if (result.detail) {
            errorMessage = result.detail;
          }
        }
        throw new Error(errorMessage);
      }
      
      // 更新本地存储中的用户信息，包括新的头像URL
      // 假设后端返回的 JSON 结构中包含新的头像 URL，例如 { avatar: 'new_url.png' }
      if (result.avatar) {
        const absoluteAvatarUrl = ensureAbsoluteAvatarUrl(result.avatar);
        const savedUserJson = localStorage.getItem('user')
        if (savedUserJson) {
           let userData = JSON.parse(savedUserJson)
           if (userData && userData.user) {
             userData.user.avatar = absoluteAvatarUrl 
             localStorage.setItem('user', JSON.stringify(userData))
           }
        }
        result.avatar = absoluteAvatarUrl;
      }

      return {
        success: true,
        message: '头像上传成功',
        data: result
      };
    } catch (error) {
      console.error('上传头像错误:', error);
      return {
        success: false,
        message: error instanceof Error ? error.message : '网络错误',
        data: null
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
          'Authorization': `Bearer ${token}`, // 使用Token认证
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

  /**
   * 从后端下载当前用户配置。
   * 接口: GET /api/sync/config/
   * @returns {Promise<{success: boolean, message: string, data: string|null}>} data 是配置文件的内容字符串。
   */
  async downloadConfig() {
    let result = null;
    try {
      const token = localStorage.getItem('token');

      if (!token) {
        throw new Error('未登录或Token缺失');
      }

      const response = await fetch(`${API_BASE_URL}/sync/config/`, {
        method: 'GET', // 使用 GET 方法下载配置
        headers: {
          'Authorization': `Bearer ${token}`, // 使用Token认证
        },
      });

      // 404 Not Found (或后端指示的特殊状态码) 表示云端没有配置文件
      if (response.status === 404) {
        return {
          success: true,
          message: '云端无配置文件',
          data: null
        };
      }
      
      // 检查响应状态是否成功 (200 OK)
      if (!response.ok) {
        // 尝试解析JSON响应以获取可能的错误信息
        try {
            result = await response.json();
        } catch (e) {
             // 忽略没有 JSON body 的情况
        }
        
        // 尝试从API响应的JSON中获取错误信息
        const errorMessage = (result && result.detail) ? result.detail : `配置下载失败，状态码: ${response.status}`;
        throw new Error(errorMessage);
      }
      
      // 成功 (200 OK)，获取配置内容 (文本格式，因为是config.json)
      const configContent = await response.text();

      return {
        success: true,
        message: '配置下载成功',
        data: configContent
      };

    } catch (error) {
      console.error('下载配置错误:', error);
      return {
        success: false,
        message: error instanceof Error ? error.message : '网络错误',
        data: null
      };
    }
  }

  /**
   * 进行ai对话。
   * 接口: POST /ai/chat/
   * @returns {Promise<{success: boolean, message: string, data: string|null}>} data 是配置文件的内容字符串。
   */
  async aiChat(chatBody, onProgress = null) {
    let result = null;
    try {
      const token = localStorage.getItem('token');

      if (!token) {
        throw new Error('未登录或Token缺失');
      }

      const response = await fetch(`${API_BASE_URL}/ai/chat/`, {
        method: 'POST', // 使用 POST 方法调用api
        headers: {
          'Authorization': `Bearer ${token}`, // 使用Token认证
        },
        body: chatBody
      });
      
      // 检查响应状态是否成功 (200 OK)
      if (!response.ok) {
        // 尝试解析JSON响应以获取可能的错误信息
        try {
            result = await response.json();
            console.log('ai API返回响应：', result)
        } catch (e) {
             // 忽略没有 JSON body 的情况
        }
        
        // 尝试从API响应的JSON中获取错误信息
        const errorMessage = (result && result.detail) ? result.detail : `AI调用失败，状态码: ${response.status}`;
        throw new Error(errorMessage);
      }
      
      // 成功 (200 OK)，获取配置内容 (文本格式，因为是config.json)
      if (onProgress) {
        // 处理流式响应
        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = '';
        let fullResponse = '';

        try {
          while (true) {
            const { done, value } = await reader.read();
            
            if (done) {
              // 流结束
              if (buffer.trim()) {
                // 处理剩余的buffer
                const finalData = this.parseSSEData(buffer);
                if (finalData) {
                  fullResponse += finalData;
                  onProgress(finalData, fullResponse);
                }
              }
              break;
            }

            // 解码数据
            const chunk = decoder.decode(value, { stream: true });
            buffer += chunk;

            // 按行分割处理 SSE 格式
            const lines = buffer.split('\n\n');
            buffer = lines.pop() || ''; // 最后一行可能不完整，留到下次处理

            for (const line of lines) {
              if (line.startsWith('data: ')) {
                const dataStr = line.substring(6); // 去掉 "data: "

                // 检查结束标记
                if (dataStr.trim() === '[DONE]') {
                  console.log('Stream finished');
                  continue;
                }

                try {
                  const data = JSON.parse(dataStr);
                  
                  if (data.type === 'delta' && data.content) {
                    // 流式文本输出
                    fullResponse += data.content;
                    // 调用进度回调
                    onProgress(data.content, fullResponse);
                  } else if (data.type === 'full' && data.content) {
                    // 完整响应
                    fullResponse = data.content;
                    onProgress(data.content, fullResponse);
                  } else if (data.type === 'error') {
                    // 错误处理
                    throw new Error(data.message || 'AI服务返回错误');
                  }
                } catch (e) {
                  console.error('Failed to parse SSE data:', e, 'Raw data:', dataStr);
                  // 如果无法解析为JSON，可能是普通文本，直接作为输出
                  if (dataStr.trim() && !dataStr.includes('[DONE]')) {
                    fullResponse += dataStr + '\n';
                    onProgress(dataStr + '\n', fullResponse);
                  }
                }
              }
            }
          }
        } finally {
          reader.releaseLock();
        }

        // 返回完整结果
        return {
          success: true,
          message: 'AI回复完成',
          data: { reply: fullResponse }
        };
      } else {
        // 非流式响应，按原方式处理
        result = await response.json();
        console.log('AI回复：', result);
        
        return {
          success: true,
          message: 'AI回复成功',
          data: result.data
        };
      }
    } catch (error) {
      console.error('下载配置错误:', error);
      return {
        success: false,
        message: error instanceof Error ? error.message : '网络错误',
        data: result.message
      };
    }
  }

  /**
   * [GET] 获取用户所有云端文件的列表。
   * 对应后端: FileListView.get
   * 接口: GET /api/sync/files/
   * @returns {Promise<{success: boolean, message: string, data: Array<Object>|null}>} data 是文件列表数组。
   */
  async getCloudFileList() {
    let result = null;
    try {
      const token = localStorage.getItem('token');
      if (!token) {
          throw new Error('未登录或Token缺失');
      }
      const response = await fetch(`${API_BASE_URL}/sync/files/`, {
          method: 'GET',
          headers: {
              'Authorization': `Bearer ${token}`,
          },
      });
      // 列表接口返回 200 OK，包含 JSON 数组
      result = await response.json();

      if (!response.ok) {
          const errorMessage = (result && result.detail) ? result.detail : '获取文件列表失败';
          throw new Error(errorMessage);
      }

      return {
          success: true,
          message: '文件列表获取成功',
          data: result
      };
    } catch (error) {
      console.error('获取文件列表错误:', error);
      return {
          success: false,
          message: error instanceof Error ? error.message : '网络错误',
          data: result
      };
    }
  }

  /**
   * [POST] 上传剪贴板文件 (支持覆盖)。
   * 对应后端: FileUploadView.post
   * 接口: POST /api/sync/files/upload/
   * @param {File | Blob} fileObject - 要上传的 File 或 Blob 对象。
   * @param {string} relativePath - 文件在云端的相对路径 (例如: 'images/screenshot.png')。
   * @returns {Promise<{success: boolean, message: string, data: object|null}>} data 是上传成功后的文件信息。
   */
  async uploadClipboardFile(fileObject, relativePath) {
    let result = null;
    try {
      const token = localStorage.getItem('token');
      if (!token) {
          throw new Error('未登录或Token缺失');
      }

      const formData = new FormData();
      // 后端 views.py 期望文件字段名为 'file'
      formData.append('file', fileObject, fileObject.name || 'clipboard_file'); 
      // 后端 views.py 期望相对路径在 request.data (FormData) 中
      formData.append('relative_path', relativePath); 

      const response = await fetch(`${API_BASE_URL}/sync/files/upload/`, {
          method: 'POST',
          headers: {
              // fetch 在使用 FormData 时会自动设置 Content-Type: multipart/form-data
              'Authorization': `Bearer ${token}`,
          },
          body: formData,
      });

      result = await response.json();

      if (!response.ok) {
          // 尝试提取错误信息
          const errorMessage = (result && result.error) ? result.error : 
                               (result && result.detail) ? result.detail : '文件上传失败';
          throw new Error(errorMessage);
      }

      return {
          success: true,
          message: '文件上传成功',
          data: result
      };

    } catch (error) {
        console.error(`上传文件 ${relativePath} 错误:`, error);
        return {
          success: false,
          message: error instanceof Error ? error.message : '网络错误',
          data: result
        };
    }
  }
  
  /**
   * [DELETE] 删除指定文件。
   * 对应后端: FileDeleteView.delete
   * 接口: DELETE /api/sync/files/{fileId}/
   * @param {number} fileId - 文件的数据库ID。
   * @returns {Promise<{success: boolean, message: string, data: null}>}
   */
  async deleteClipboardFile(fileId) {
    let result = null;
    try {
      const token = localStorage.getItem('token');
      if (!token) {
          throw new Error('未登录或Token缺失');
      }
      
      const response = await fetch(`${API_BASE_URL}/sync/files/${fileId}/`, {
          method: 'DELETE',
          headers: {
              'Authorization': `Bearer ${token}`,
          },
      });

      // 成功删除返回 204 No Content
      if (response.status === 204) {
          return {
              success: true,
              message: `文件 ID: ${fileId} 删除成功`,
              data: null
          };
      }
      
      // 尝试解析JSON响应以获取可能的错误信息
      try {
          result = await response.json();
      } catch (e) {
          // 忽略没有 JSON body 的情况
      }

      // 非 204 且非 ok 视为失败
      if (!response.ok) {
          const errorMessage = (result && result.detail) ? result.detail : 
                               `文件删除失败，状态码: ${response.status}`;
          throw new Error(errorMessage);
      }
            
      // 理论上不会走到这里，但以防万一
      return {
          success: false, 
          message: '文件删除失败或响应异常', 
          data: result 
      };

    } catch (error) {
      console.error(`删除文件 ID: ${fileId} 错误:`, error);
      return {
        success: false,
        message: error instanceof Error ? error.message : '网络错误',
        data: result
      };
    }
  }

  /**
   * [POST] 上传 SQLite 数据库文件并同步数据到服务器。
   * 对应后端: SqlitePushView.post
   * 接口: POST /api/sync/sqlite/push/
   * @param {File | Blob} dbFile - SQLite 数据库文件对象。
   * @returns {Promise<{success: boolean, message: string, data: object|null}>}
   */
  async pushSqliteDatabase(dbFile) {
    let result = null;
    try {
      const token = localStorage.getItem('token');
      if (!token) {
          throw new Error('未登录或Token缺失');
      }

      const formData = new FormData();
      // 后端 views.py 期望文件字段名为 'db_file'
      formData.append('db_file', dbFile, dbFile.name || 'clipboard_sync.db'); 

      const response = await fetch(`${API_BASE_URL}/sync/sqlite/push/`, {
          method: 'POST',
          headers: {
              // fetch 在使用 FormData 时会自动设置 Content-Type: multipart/form-data
              'Authorization': `Bearer ${token}`,
          },
          body: formData,
      });

      result = await response.json();

      if (!response.ok) {
        const errorMessage = (result && result.error) ? result.error : 
                             (result && result.detail) ? result.detail : '数据库推送失败';
        throw new Error(errorMessage);
      }

      return {
        success: true,
        message: '数据库同步成功',
        data: result
      };

    } catch (error) {
      console.error('推送 SQLite 数据库错误:', error);
      return {
        success: false,
        message: error instanceof Error ? error.message : '网络错误',
        data: result
      };
    }
  }

  /**
   * [GET] 获取当前用户的 SQLite 数据库文件内容，并返回 json 格式的数据。
   * 对应后端: SqliteGetView.get
   * 接口: GET /api/sync/sqlite/get/
   * @returns {Promise<{success: boolean, message: string, data: object|null}>} data 包含数据库数据（json格式）
   */
  async getSqliteDatabaseAsJson() {
    let result = null;
    try {
      const token = localStorage.getItem('token');
      if (!token) {
        throw new Error('未登录或Token缺失');
      }
      
      const response = await fetch(`${API_BASE_URL}/sync/sqlite/get/`, {
        method: 'GET',
        headers: {
            'Authorization': `Bearer ${token}`,
        },
      });

      result = await response.json();

      if (!response.ok) {
          const errorMessage = (result && result.error) ? result.error : 
                               (result && result.detail) ? result.detail : '获取 SQLite 数据(JSON)失败';
          throw new Error(errorMessage);
      }
      
      // 后端返回的结构是 { message: "...", data: user_data }，我们只需要 user_data
      return {
          success: true,
          message: 'SQLite 数据获取成功',
          data: result.data 
      };

    } catch (error) {
      console.error('获取 SQLite 数据(JSON)错误:', error);
      return {
          success: false,
          message: error instanceof Error ? error.message : '网络错误',
          data: result
      };
    }
  }

  /**
   * 解析SSE数据
   * @param {string} dataStr - SSE数据字符串
   * @returns {string|null} 解析出的内容
   */
  parseSSEData(dataStr) {
    if (!dataStr.trim() || dataStr.trim() === '[DONE]') {
      return null;
    }
    
    try {
      const data = JSON.parse(dataStr);
      if (data.type === 'delta' && data.content) {
        return data.content;
      } else if (data.type === 'full' && data.content) {
        return data.content;
      }
    } catch (e) {
      // 如果不是JSON，直接返回原始文本
      return dataStr;
    }
    
    return null;
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

