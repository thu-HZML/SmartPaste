// src/components/MenuFunctions.js
import { ref,onMounted, onUnmounted } from "vue";
import { emit, listen } from '@tauri-apps/api/event'
import { toggleClipboardWindow, toggleFavoritesWindow, toggleSetWindow } from '../utils/actions.js'

const username = ref("未登录");
const userAvatar = ref("");

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
        
        // 读取用户头像URL
        if (userData.user.avatar) {
          userAvatar.value = userData.user.avatar;
          console.log(`用户头像URL: ${userAvatar.value}`);
        } else {
          // 如果用户数据中没有头像，尝试从其他地方获取
          const savedAvatar = localStorage.getItem("userAvatar");
          if (savedAvatar) {
            userAvatar.value = savedAvatar;
            console.log(`从独立存储加载头像URL: ${userAvatar.value}`);
          } else {
            userAvatar.value = ""; // 设置为空字符串，显示默认图标
            console.log('未找到用户头像数据');
          }
        }

        return; 
      }
    } catch (e) {
      console.error("解析本地存储的 'user' 数据失败:", e);
    }
  }
  
  // 失败或未登录，则设置为默认值
  username.value = "未登录";
  userAvatar.value = "";
  console.log('未找到有效用户名数据，设置为: 未登录');
}
// 初始化时加载用户名
loadUsername();

export function useUsername() {
  let unlisten = null;

  onMounted(async () => {
    // 1. 每次组件挂载时，确保数据是最新的
    loadUsername();

    // 2. 注册全局事件监听器
    // 当其他窗口（如设置页）发出 'user-info-updated' 事件时，刷新数据
    unlisten = await listen('user-info-updated', () => {
      console.log('收到用户信息更新通知，正在刷新 Menu...');
      loadUsername();
    });
  });

  onUnmounted(() => {
    // 组件卸载时取消监听，防止内存泄漏
    if (unlisten) {
      unlisten();
    }
  });
  
  return {
    username, // 返回全局的响应式引用
    userAvatar,
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

  const openSettings = async (navId = 'general') => {
    console.log(`尝试打开设置，目标子页面: ${navId}`)
    try {
      await toggleSetWindow(navId)
      console.log('设置窗口已打开')
    } catch (error) {
      console.error('打开设置失败:', error)
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