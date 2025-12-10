// src/components/MenuFunctions.js
import { ref } from "vue";
import { toggleClipboardWindow, toggleFavoritesWindow } from '../utils/actions.js'

const username = ref("未登录");

export function loadUsername() {
  console.log('尝试从本地存储加载用户名...');
  // 读取存储的 'user' 键下的 JSON 字符串
  const storedUserJson = localStorage.getItem("user"); 

  if (storedUserJson) {
    
    try {
      const userData = JSON.parse(storedUserJson);
      console.log('读取到的 userData:', userData);
      // 访问 userData 中的 user.username 字段
      if (userData && userData.user ) {
        username.value = userData.user.username; // 更新全局 ref 的值
        console.log(`用户名已更新为: ${username.value}`);
        return; 
      }
    } catch (e) {
      console.error("解析本地存储的 'user' 数据失败:", e);
    }
  }
  
  // 失败或未登录，则设置为默认值
  username.value = "未登录";
  console.log('未找到有效用户名数据，设置为: 未登录');
}
loadUsername();

export function useUsername() {
  return {
    username, // 返回全局的响应式引用
    loadUsername,
  };
}

export function useMenuFunctions() {
  const openHistory = async () => {
    console.log('打开历史记录 - 调用 toggleClipboardWindow')
    try {
      await toggleClipboardWindow()
      console.log('📋 剪贴板窗口已切换')
    } catch (error) {
      console.error('切换剪贴板窗口失败:', error)
    }
  }

  const openFavorites = async () => {
    console.log('打开收藏夹')
    try {
      await toggleFavoritesWindow()
      console.log('⭐ 收藏夹窗口已打开')
    } catch (error) {
      console.error('打开收藏夹窗口失败:', error)
    }
  }

  const openSettings = async () => {
    console.log('打开设置')
    try {
      await toggleSetWindow()
      console.log('设置窗口已打开')
    } catch (error) {
      console.error('打开设置窗口失败:', error)
    }
  }

  const openCloud = () => {
    console.log('打开云端')
  }

  const openAIAssistant = () => {
    console.log('打开AI助手')
  }

  const openHelp = () => {
    console.log('打开帮助')
  }

  return {
    openHistory,
    openFavorites,
    openSettings,
    openCloud,
    openAIAssistant,
    openHelp
  }
}