<template>
  <div class="menu-container">
    <!-- 菜单头部 -->
    <header class="menu-header">
      <div class="user-section">
        <div class="user-avatar">
          <img 
            src="https://ide.code.fun/api/image?token=69034a079520a30011f4f4f9&name=f8435267bedb1f8da2ed89ce0b7f6027.png" 
            alt="用户头像"
            class="avatar-img"
          />
        </div>
        <div class="user-info">
          <h3 class="username">当前用户</h3>
          <p class="user-status">已登录</p>
        </div>
      </div>
    </header>

    <!-- 菜单内容 -->
    <main class="menu-content">
      <!-- 主要功能区 -->
      <div class="menu-section">
        <h4 class="section-title">主要功能</h4>
        <div class="menu-grid">
          <button class="menu-item" @click="goToClipboard">
            <span class="menu-icon">📋</span>
            <span class="menu-text">剪贴板管理</span>
            <span class="menu-arrow">→</span>
          </button>
          
          <button class="menu-item" @click="goToAI">
            <span class="menu-icon">🤖</span>
            <span class="menu-text">AI 交互</span>
            <span class="menu-arrow">→</span>
          </button>
          
          <button class="menu-item" @click="goToSettings">
            <span class="menu-icon">⚙️</span>
            <span class="menu-text">系统设置</span>
            <span class="menu-arrow">→</span>
          </button>
        </div>
      </div>

      <!-- 常用设置 -->
      <div class="menu-section">
        <h4 class="section-title">常用设置</h4>
        <div class="menu-grid">
          <button class="menu-item" @click="goToSetting('cloud')">
            <span class="menu-icon">☁️</span>
            <span class="menu-text">云端同步</span>
            <span class="menu-badge" v-if="settings.cloudSync">已开启</span>
          </button>
          
          <button class="menu-item" @click="goToSetting('shortcuts')">
            <span class="menu-icon">⌨️</span>
            <span class="menu-text">快捷键设置</span>
            <span class="menu-badge">{{ settings.shortcuts.toggleWindow || '未设置' }}</span>
          </button>
          
          <button class="menu-item" @click="goToSetting('general')">
            <span class="menu-icon">📝</span>
            <span class="menu-text">剪贴板参数</span>
            <span class="menu-badge">{{ settings.maxHistoryItems }}条</span>
          </button>
          
          <button class="menu-item" @click="toggleAutoStart">
            <span class="menu-icon">🚀</span>
            <span class="menu-text">开机自启</span>
            <label class="toggle-switch mini">
              <input type="checkbox" v-model="settings.autoStart">
              <span class="slider"></span>
            </label>
          </button>
          
          <button class="menu-item" @click="toggleTrayIcon">
            <span class="menu-icon">📌</span>
            <span class="menu-text">托盘图标</span>
            <label class="toggle-switch mini">
              <input type="checkbox" v-model="settings.showTrayIcon">
              <span class="slider"></span>
            </label>
          </button>
        </div>
      </div>

      <!-- 快速操作 -->
      <div class="menu-section">
        <h4 class="section-title">快速操作</h4>
        <div class="quick-actions">
          <button class="quick-btn" @click="clearHistory">
            <span class="quick-icon">🗑️</span>
            <span class="quick-text">清空历史</span>
          </button>
          
          <button class="quick-btn" @click="exportData">
            <span class="quick-icon">📤</span>
            <span class="quick-text">导出数据</span>
          </button>
          
          <button class="quick-btn" @click="importData">
            <span class="quick-icon">📥</span>
            <span class="quick-text">导入数据</span>
          </button>
        </div>
      </div>
    </main>

    <!-- 底部状态 -->
    <footer class="menu-footer">
      <div class="status-info">
        <span class="status-item">历史记录: {{ historyCount }} 条</span>
        <span class="status-item">收藏: {{ favoriteCount }} 个</span>
      </div>
      <button class="logout-btn" @click="logout">
        <span class="logout-icon">🚪</span>
        退出
      </button>
    </footer>

    <!-- 提示信息 -->
    <div v-if="showToast" class="toast">
      {{ toastMessage }}
    </div>
  </div>
</template>

<script>
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'

export default {
  name: 'MainMenu',
  setup() {
    const router = useRouter()
    
    const showToast = ref(false)
    const toastMessage = ref('')
    
    const settings = reactive({
      autoStart: true,
      showTrayIcon: true,
      cloudSync: false,
      maxHistoryItems: 100,
      shortcuts: {
        toggleWindow: 'Ctrl+Shift+V',
        quickPaste: '',
        clearHistory: ''
      }
    })
    
    const historyCount = ref(42)
    const favoriteCount = ref(8)

    // 导航功能
    const goToClipboard = () => {
      router.push('/')
      showMessage('跳转到剪贴板')
    }
    
    const goToAI = () => {
      showMessage('AI 交互功能开发中')
    }
    
    const goToSettings = () => {
      router.push('/preferences')
      showMessage('跳转到设置')
    }
    
    const goToSetting = (section) => {
      router.push(`/preferences?section=${section}`)
      showMessage(`跳转到${getSectionName(section)}`)
    }
    
    const getSectionName = (section) => {
      const names = {
        'cloud': '云端同步',
        'shortcuts': '快捷键设置',
        'clipboard': '剪贴板参数'
      }
      return names[section] || '设置'
    }

    // 设置切换
    const toggleAutoStart = () => {
      settings.autoStart = !settings.autoStart
      showMessage(settings.autoStart ? '已开启开机自启' : '已关闭开机自启')
    }
    
    const toggleTrayIcon = () => {
      settings.showTrayIcon = !settings.showTrayIcon
      showMessage(settings.showTrayIcon ? '已显示托盘图标' : '已隐藏托盘图标')
    }

    // 快速操作
    const clearHistory = () => {
      if (confirm('确定要清空所有历史记录吗？此操作不可撤销！')) {
        historyCount.value = 0
        showMessage('历史记录已清空')
      }
    }
    
    const exportData = () => {
      showMessage('数据导出功能开发中')
    }
    
    const importData = () => {
      showMessage('数据导入功能开发中')
    }
    
    const logout = () => {
      if (confirm('确定要退出应用吗？')) {
        showMessage('正在退出应用...')
        // 实际应用中这里会调用退出逻辑
      }
    }

    // 工具函数
    const showMessage = (message) => {
      toastMessage.value = message
      showToast.value = true
      setTimeout(() => {
        showToast.value = false
      }, 2000)
    }

    onMounted(() => {
      // 加载设置
      const savedSettings = localStorage.getItem('clipboardSettings')
      if (savedSettings) {
        Object.assign(settings, JSON.parse(savedSettings))
      }
    })

    return {
      settings,
      historyCount,
      favoriteCount,
      showToast,
      toastMessage,
      goToClipboard,
      goToAI,
      goToSettings,
      goToSetting,
      toggleAutoStart,
      toggleTrayIcon,
      clearHistory,
      exportData,
      importData,
      logout
    }
  }
}
</script>

<style scoped>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

.menu-container {
  min-height: 100vh;
  background: white;
  display: flex;
  flex-direction: column;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  overflow-x: hidden;
  max-width: 100%;
}

/* 菜单头部 */
.menu-header {
  padding: 16px 20px;
  border-bottom: 1px solid #e1e8ed;
  background: white;
}

.user-section {
  display: flex;
  align-items: center;
  gap: 12px;
}

.user-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  overflow: hidden;
  border: 2px solid #e1e8ed;
}

.avatar-img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.user-info {
  flex: 1;
}

.username {
  font-size: 16px;
  font-weight: 600;
  color: #2c3e50;
  margin-bottom: 2px;
}

.user-status {
  font-size: 12px;
  color: #7f8c8d;
}

/* 菜单内容 */
.menu-content {
  flex: 1;
  padding: 16px 20px;
  overflow-y: auto;
}

.menu-section {
  margin-bottom: 24px;
}

.section-title {
  font-size: 14px;
  font-weight: 600;
  color: #7f8c8d;
  margin-bottom: 12px;
  padding-left: 8px;
}

.menu-grid {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.menu-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  border: none;
  border-radius: 8px;
  background: white;
  cursor: pointer;
  transition: all 0.2s ease;
  text-align: left;
  width: 100%;
}

.menu-item:hover {
  background: #f8f9fa;
  border-color: #b7c8fe;
}

.menu-item:active {
  transform: translateY(1px);
}

.menu-icon {
  font-size: 18px;
  margin-right: 12px;
  width: 24px;
  text-align: center;
}

.menu-text {
  flex: 1;
  font-size: 14px;
  color: #2c3e50;
  font-weight: 500;
}

.menu-arrow {
  color: #bdc3c7;
  font-size: 14px;
}

.menu-badge {
  background: #edf3fe;
  color: #3498db;
  padding: 4px 8px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 500;
}

/* 切换开关迷你版 */
.toggle-switch.mini {
  width: 36px;
  height: 20px;
}

.toggle-switch.mini .slider:before {
  height: 14px;
  width: 14px;
  left: 3px;
  bottom: 3px;
}

.toggle-switch.mini input:checked + .slider:before {
  transform: translateX(16px);
}

/* 快速操作 */
.quick-actions {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 8px;
}

.quick-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 12px 8px;
  border: 1px solid #e1e8ed;
  border-radius: 8px;
  background: white;
  cursor: pointer;
  transition: all 0.2s ease;
}

.quick-btn:hover {
  border-color: #b7c8fe;
  background: #f8f9fa;
}

.quick-icon {
  font-size: 20px;
  margin-bottom: 4px;
}

.quick-text {
  font-size: 12px;
  color: #2c3e50;
  font-weight: 500;
}

/* 菜单底部 */
.menu-footer {
  padding: 16px 20px;
  border-top: 1px solid #e1e8ed;
  background: white;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.status-info {
  display: flex;
  gap: 16px;
}

.status-item {
  font-size: 12px;
  color: #7f8c8d;
}

.logout-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  border: 1px solid #e1e8ed;
  border-radius: 6px;
  background: white;
  color: #e74c3c;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.logout-btn:hover {
  background: #fdf2f2;
  border-color: #e74c3c;
}

.logout-icon {
  font-size: 14px;
}

/* 提示信息 */
.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.8);
  color: white;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 14px;
  z-index: 1000;
  animation: slideUp 0.3s ease;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

/* 切换开关样式（复用设置界面的样式） */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 44px;
  height: 24px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: #ccc;
  transition: .4s;
  border-radius: 24px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 18px;
  width: 18px;
  left: 3px;
  bottom: 3px;
  background-color: white;
  transition: .4s;
  border-radius: 50%;
}

input:checked + .slider {
  background-color: #3498db;
}

input:checked + .slider:before {
  transform: translateX(20px);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .menu-header {
    padding: 12px 16px;
  }
  
  .menu-content {
    padding: 12px 16px;
  }
  
  .menu-footer {
    padding: 12px 16px;
    flex-direction: column;
    gap: 12px;
    align-items: stretch;
  }
  
  .status-info {
    justify-content: space-between;
  }
  
  .quick-actions {
    grid-template-columns: repeat(3, 1fr);
  }
}
</style>