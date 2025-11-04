<template>
  <div class="settings-container">
    <!-- 设置头部 -->
    <header class="settings-header">
      <h1>设置</h1>
      <button class="back-btn" @click="goBack">← 返回</button>
    </header>

    <!-- 设置内容区域 -->
    <div class="settings-content">
      <!-- 左侧导航栏 -->
      <nav class="settings-nav">
        <ul class="nav-list">
          <li 
            v-for="item in navItems" 
            :key="item.id"
            :class="['nav-item', { active: activeNav === item.id }]"
            @click="setActiveNav(item.id)"
          >
            <span class="nav-icon">{{ item.icon }}</span>
            <span class="nav-text">{{ item.name }}</span>
          </li>
        </ul>
      </nav>

      <!-- 右侧设置面板 -->
      <div class="settings-panel">
        <!-- 通用设置 -->
        <div v-if="activeNav === 'general'" class="panel-section">
          <h2>通用设置</h2>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>启动时自动运行</h3>
              <p>开机时自动启动剪贴板管理器</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input type="checkbox" v-model="settings.autoStart">
                <span class="slider"></span>
              </label>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>显示系统托盘图标</h3>
              <p>在系统托盘显示应用图标，方便快速访问</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input type="checkbox" v-model="settings.showTrayIcon">
                <span class="slider"></span>
              </label>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>自动保存剪贴板历史</h3>
              <p>自动保存剪贴板内容到历史记录</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input type="checkbox" v-model="settings.autoSave">
                <span class="slider"></span>
              </label>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>历史记录保留时间</h3>
              <p>自动删除超过指定天数的历史记录</p>
            </div>
            <div class="setting-control">
              <select v-model="settings.retentionDays" class="select-input">
                <option value="7">7天</option>
                <option value="30">30天</option>
                <option value="90">90天</option>
                <option value="0">永久保存</option>
              </select>
            </div>
          </div>
        </div>

        <!-- 快捷键设置 -->
        <div v-if="activeNav === 'shortcuts'" class="panel-section">
          <h2>快捷键设置</h2>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>显示/隐藏主窗口</h3>
              <p>快速显示或隐藏剪贴板管理器主窗口</p>
            </div>
            <div class="setting-control">
              <div class="shortcut-input" @click="startRecording('toggleWindow')">
                {{ settings.shortcuts.toggleWindow || '点击设置' }}
              </div>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>快速粘贴</h3>
              <p>使用快捷键快速粘贴最近的内容</p>
            </div>
            <div class="setting-control">
              <div class="shortcut-input" @click="startRecording('quickPaste')">
                {{ settings.shortcuts.quickPaste || '点击设置' }}
              </div>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>清空剪贴板历史</h3>
              <p>快速清空所有剪贴板历史记录</p>
            </div>
            <div class="setting-control">
              <div class="shortcut-input" @click="startRecording('clearHistory')">
                {{ settings.shortcuts.clearHistory || '点击设置' }}
              </div>
            </div>
          </div>
          
          <div class="hint">
            <p>提示：点击快捷键输入框，然后按下您想要设置的组合键</p>
          </div>
        </div>

        <!-- 剪贴板参数设置 -->
        <div v-if="activeNav === 'clipboard'" class="panel-section">
          <h2>剪贴板参数设置</h2>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>最大历史记录数量</h3>
              <p>限制保存的剪贴板历史记录数量</p>
            </div>
            <div class="setting-control">
              <input 
                type="number" 
                v-model="settings.maxHistoryItems" 
                min="10" 
                max="1000" 
                class="number-input"
              >
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>忽略短文本</h3>
              <p>不保存字符数少于指定值的文本</p>
            </div>
            <div class="setting-control">
              <input 
                type="number" 
                v-model="settings.ignoreShortText" 
                min="0" 
                max="50" 
                class="number-input"
              >
              <span class="unit">字符</span>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>忽略特定应用</h3>
              <p>不记录来自这些应用的剪贴板内容</p>
            </div>
            <div class="setting-control">
              <div class="tag-input-container">
                <div 
                  v-for="(app, index) in settings.ignoredApps" 
                  :key="index" 
                  class="tag"
                >
                  {{ app }}
                  <span @click="removeIgnoredApp(index)" class="tag-remove">×</span>
                </div>
                <input 
                  type="text" 
                  v-model="newIgnoredApp" 
                  placeholder="输入应用名称" 
                  @keyup.enter="addIgnoredApp"
                  class="tag-input"
                >
              </div>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>文本预览长度</h3>
              <p>在列表中显示的文本预览长度</p>
            </div>
            <div class="setting-control">
              <input 
                type="number" 
                v-model="settings.previewLength" 
                min="20" 
                max="200" 
                class="number-input"
              >
              <span class="unit">字符</span>
            </div>
          </div>
        </div>

        <!-- 云端入口 -->
        <div v-if="activeNav === 'cloud'" class="panel-section">
          <h2>云端同步</h2>
          
          <div class="setting-item">
            <div class="setting-info">
              <h3>启用云端同步</h3>
              <p>将剪贴板历史同步到云端，跨设备访问</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input type="checkbox" v-model="settings.cloudSync">
                <span class="slider"></span>
              </label>
            </div>
          </div>
          
          <div v-if="settings.cloudSync" class="cloud-settings">
            <div class="setting-item">
              <div class="setting-info">
                <h3>同步频率</h3>
                <p>自动同步剪贴板历史的频率</p>
              </div>
              <div class="setting-control">
                <select v-model="settings.syncFrequency" class="select-input">
                  <option value="realtime">实时同步</option>
                  <option value="5min">每5分钟</option>
                  <option value="15min">每15分钟</option>
                  <option value="1hour">每小时</option>
                </select>
              </div>
            </div>
            
            <div class="setting-item">
              <div class="setting-info">
                <h3>加密同步数据</h3>
                <p>使用端到端加密保护您的剪贴板数据</p>
              </div>
              <div class="setting-control">
                <label class="toggle-switch">
                  <input type="checkbox" v-model="settings.encryptCloudData">
                  <span class="slider"></span>
                </label>
              </div>
            </div>
            
            <div class="account-status" v-if="!userLoggedIn">
              <p>您尚未登录，请登录以启用云端同步功能</p>
              <button class="btn btn-primary" @click="login">登录账户</button>
            </div>
            
            <div class="account-status" v-else>
              <p>已登录为: {{ userEmail }}</p>
              <button class="btn btn-secondary" @click="logout">退出登录</button>
            </div>
          </div>
        </div>

        <!-- 用户信息 -->
        <div v-if="activeNav === 'user'" class="panel-section">
          <h2>用户信息</h2>
          
          <div class="user-profile">
            <div class="avatar-section">
              <div class="avatar">👤</div>
              <button class="btn btn-secondary">更换头像</button>
            </div>
            
            <div class="user-details">
              <div class="form-group">
                <label>用户名</label>
                <input type="text" v-model="userInfo.username" class="text-input">
              </div>
              
              <div class="form-group">
                <label>电子邮箱</label>
                <input type="email" v-model="userInfo.email" class="text-input">
              </div>
              
              <div class="form-group">
                <label>个人简介</label>
                <textarea v-model="userInfo.bio" class="textarea-input" rows="3"></textarea>
              </div>
              
              <div class="form-actions">
                <button class="btn btn-primary" @click="saveUserInfo">保存更改</button>
                <button class="btn btn-secondary" @click="resetUserInfo">重置</button>
              </div>
            </div>
          </div>
          
          <div class="account-actions">
            <h3>账户操作</h3>
            <div class="action-buttons">
              <button class="btn btn-secondary" @click="changePassword">修改密码</button>
              <button class="btn btn-danger" @click="deleteAccount">删除账户</button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- 提示信息 -->
    <div v-if="showToast" class="toast">
      {{ toastMessage }}
    </div>
  </div>
</template>

<script>
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'

export default {
  name: 'Settings',
  setup() {
    const router = useRouter()
    
    const activeNav = ref('general')
    const showToast = ref(false)
    const toastMessage = ref('')
    const recordingShortcut = ref('')
    const newIgnoredApp = ref('')
    const userLoggedIn = ref(false)
    const userEmail = ref('user@example.com')
    
    const navItems = [
      { id: 'general', name: '通用设置', icon: '⚙️' },
      { id: 'shortcuts', name: '快捷键设置', icon: '⌨️' },
      { id: 'clipboard', name: '剪贴板参数设置', icon: '📋' },
      { id: 'cloud', name: '云端入口', icon: '☁️' },
      { id: 'user', name: '用户信息', icon: '👤' }
    ]
    
    const settings = reactive({
      autoStart: true,
      showTrayIcon: true,
      autoSave: true,
      retentionDays: '30',
      maxHistoryItems: 100,
      ignoreShortText: 3,
      ignoredApps: ['密码管理器', '银行应用'],
      previewLength: 115,
      cloudSync: false,
      syncFrequency: 'realtime',
      encryptCloudData: true,
      shortcuts: {
        toggleWindow: 'Ctrl+Shift+V',
        quickPaste: '',
        clearHistory: ''
      }
    })
    
    const userInfo = reactive({
      username: '当前用户',
      email: 'user@example.com',
      bio: '剪贴板管理爱好者'
    })
    
    const setActiveNav = (navId) => {
      activeNav.value = navId
    }
    
    const goBack = () => {
      router.back()
    }
    
    const startRecording = (shortcutName) => {
      recordingShortcut.value = shortcutName
      showMessage('请按下快捷键组合...')
      // 这里应该添加键盘事件监听器来捕获按键
      // 简化实现，仅作演示
      setTimeout(() => {
        settings.shortcuts[shortcutName] = 'Ctrl+Shift+' + shortcutName.charAt(0).toUpperCase()
        recordingShortcut.value = ''
        showMessage('快捷键已设置')
      }, 1000)
    }
    
    const addIgnoredApp = () => {
      if (newIgnoredApp.value.trim() && !settings.ignoredApps.includes(newIgnoredApp.value.trim())) {
        settings.ignoredApps.push(newIgnoredApp.value.trim())
        newIgnoredApp.value = ''
        showMessage('已添加忽略应用')
      }
    }
    
    const removeIgnoredApp = (index) => {
      settings.ignoredApps.splice(index, 1)
      showMessage('已移除忽略应用')
    }
    
    const login = () => {
      // 模拟登录
      userLoggedIn.value = true
      showMessage('登录成功')
    }
    
    const logout = () => {
      userLoggedIn.value = false
      showMessage('已退出登录')
    }
    
    const saveUserInfo = () => {
      showMessage('用户信息已保存')
    }
    
    const resetUserInfo = () => {
      Object.assign(userInfo, {
        username: '当前用户',
        email: 'user@example.com',
        bio: '剪贴板管理爱好者'
      })
      showMessage('用户信息已重置')
    }
    
    const changePassword = () => {
      showMessage('修改密码功能待实现')
    }
    
    const deleteAccount = () => {
      if (confirm('确定要删除账户吗？此操作不可撤销！')) {
        showMessage('账户删除功能待实现')
      }
    }
    
    const showMessage = (message) => {
      toastMessage.value = message
      showToast.value = true
      setTimeout(() => {
        showToast.value = false
      }, 2000)
    }
    
    onMounted(() => {
      // 加载保存的设置
      const savedSettings = localStorage.getItem('clipboardSettings')
      if (savedSettings) {
        Object.assign(settings, JSON.parse(savedSettings))
      }
    })
    
    return {
      activeNav,
      navItems,
      settings,
      userInfo,
      showToast,
      toastMessage,
      newIgnoredApp,
      userLoggedIn,
      userEmail,
      setActiveNav,
      goBack,
      startRecording,
      addIgnoredApp,
      removeIgnoredApp,
      login,
      logout,
      saveUserInfo,
      resetUserInfo,
      changePassword,
      deleteAccount
    }
  }
}
</script>

<style scoped>
* {
  box-sizing: border-box;
}

.settings-container {
  min-height: 100vh;
  background: white;
  overflow-x: hidden;
  max-width: 100%;
  width: 100vw;
  position: fixed;
}

/* 设置头部样式 */
.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 8px;
  border-bottom: 1px solid #e1e8ed;
  background: white;
  max-width: 100%;
}

.settings-header h1 {
  font-size: 15px;
  font-weight: 600;
  color: #2c3e50;
}

.back-btn {
  padding: 6px 8px;
  border: 1px solid #e1e8ed;
  border-radius: 6px;
  background: white;
  color: #3498db;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.back-btn:hover {
  background: #f8f9fa;
  border-color: #3498db;
}

/* 设置内容区域 */
.settings-content {
  display: flex;
  height: calc(100vh - 40px);
  max-width: 100%;
}

/* 左侧导航栏 */
.settings-nav {
  width: 200px;
  border-right: 1px solid #e1e8ed;
  background: #f8f9fa;
  overflow-y: auto;
  padding: 6px 8px;
}

.nav-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.nav-item {
  display: flex;
  align-items: center;
  padding: 12px 16px;
  cursor: pointer;
  transition: all 0.1s;
  border: none;
  border-radius: 8px;
}

.nav-item:hover {
  background: #f1f3f5;
}

.nav-item.active {
  background: #e4edfd;
  color: #416afe;
}

.nav-icon {
  margin-right: 12px;
  font-size: 16px;
}

.nav-text {
  font-size: 14px;
  font-weight: 500;
}

/* 右侧设置面板 */
.settings-panel {
  flex: 1;
  padding: 24px;
  overflow-y: auto;
  background: white;
}

.panel-section h2 {
  margin-bottom: 24px;
  font-size: 20px;
  font-weight: 600;
  color: #2c3e50;
  border-bottom: 1px solid #e1e8ed;
  padding-bottom: 12px;
}

.setting-item {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  padding: 16px 0;
  border-bottom: 1px solid #f0f0f0;
}

.setting-info h3 {
  margin: 0 0 4px 0;
  font-size: 15px;
  font-weight: 500;
  color: #2c3e50;
}

.setting-info p {
  margin: 0;
  font-size: 13px;
  color: #7f8c8d;
}

.setting-control {
  display: flex;
  align-items: center;
  min-width: 160px;
  justify-content: flex-end;
}

/* 切换开关样式 */
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

/* 输入框样式 */
.select-input, .number-input, .text-input, .textarea-input {
  padding: 8px 12px;
  border: 1px solid #e1e8ed;
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.select-input:focus, .number-input:focus, .text-input:focus, .textarea-input:focus {
  border-color: #3498db;
}

.number-input {
  width: 80px;
}

.text-input, .textarea-input {
  width: 100%;
}

.unit {
  margin-left: 8px;
  font-size: 14px;
  color: #7f8c8d;
}

/* 标签输入样式 */
.tag-input-container {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
  min-width: 200px;
}

.tag {
  display: flex;
  align-items: center;
  background: #edf3fe;
  color: #3498db;
  padding: 4px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.tag-remove {
  margin-left: 4px;
  cursor: pointer;
  font-weight: bold;
}

.tag-input {
  flex: 1;
  min-width: 120px;
  padding: 4px 8px;
  border: 1px solid #e1e8ed;
  border-radius: 4px;
  font-size: 12px;
}

/* 快捷键输入样式 */
.shortcut-input {
  padding: 8px 12px;
  border: 1px solid #e1e8ed;
  border-radius: 6px;
  background: white;
  cursor: pointer;
  text-align: center;
  min-width: 120px;
  transition: all 0.2s;
}

.shortcut-input:hover {
  border-color: #3498db;
  background: #f8f9fa;
}

.hint {
  margin-top: 24px;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 6px;
  font-size: 13px;
  color: #7f8c8d;
}

/* 云端设置样式 */
.cloud-settings {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #f0f0f0;
}

.account-status {
  margin-top: 24px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
  text-align: center;
}

.account-status p {
  margin-bottom: 12px;
  font-size: 14px;
  color: #2c3e50;
}

/* 用户信息样式 */
.user-profile {
  display: flex;
  gap: 24px;
  margin-bottom: 32px;
}

.avatar-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.avatar {
  width: 80px;
  height: 80px;
  border-radius: 50%;
  background: #edf3fe;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
}

.user-details {
  flex: 1;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: #2c3e50;
}

.form-actions {
  display: flex;
  gap: 12px;
  margin-top: 24px;
}

.account-actions {
  padding-top: 24px;
  border-top: 1px solid #f0f0f0;
}

.account-actions h3 {
  margin-bottom: 16px;
  font-size: 16px;
  font-weight: 500;
  color: #2c3e50;
}

.action-buttons {
  display: flex;
  gap: 12px;
}

/* 按钮样式 */
.btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: #3498db;
  color: white;
}

.btn-primary:hover {
  background: #2980b9;
}

.btn-secondary {
  background: #ecf0f1;
  color: #2c3e50;
  border: 1px solid #bdc3c7;
}

.btn-secondary:hover {
  background: #d5dbdb;
}

.btn-danger {
  background: #e74c3c;
  color: white;
}

.btn-danger:hover {
  background: #c0392b;
}

/* 提示信息样式 */
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

/* 响应式设计 */
@media (max-width: 768px) {
  .settings-content {
    flex-direction: column;
    height: auto;
  }
  
  .settings-nav {
    width: 100%;
    border-right: none;
    border-bottom: 1px solid #e1e8ed;
  }
  
  .nav-list {
    display: flex;
    overflow-x: auto;
  }
  
  .nav-item {
    flex-shrink: 0;
    border-left: none;
    border-bottom: 3px solid transparent;
  }
  
  .nav-item.active {
    border-left-color: transparent;
    border-bottom-color: #3498db;
  }
  
  .setting-item {
    flex-direction: column;
    align-items: flex-start;
  }
  
  .setting-control {
    margin-top: 12px;
    width: 100%;
    justify-content: flex-start;
  }
  
  .user-profile {
    flex-direction: column;
  }
  
  .avatar-section {
    align-self: center;
  }
}
</style>