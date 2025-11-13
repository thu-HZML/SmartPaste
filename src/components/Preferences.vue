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
            <component :is="item.icon" class="nav-icon" />
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
                <input type="checkbox" v-model="settings.autoStart" @change="toggleAutoStart">
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
              <h3>显示/隐藏剪贴板</h3>
              <p>快速显示或隐藏剪贴板</p>
            </div>
            <div class="setting-control">
              <div class="shortcut-input" @click="startRecording2('pasteWindow')">
                {{ settings.shortcuts.pasteWindow || '点击设置' }}
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

<script setup>
import { ref, reactive, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { 
  Cog6ToothIcon,
  TvIcon,
  CloudIcon,
  ClipboardIcon,
  UserIcon
 } from '@heroicons/vue/24/outline'

const router = useRouter()

// 响应式数据
const activeNav = ref('general')
const showToast = ref(false)
const toastMessage = ref('')
const recordingShortcut = ref('')
const newIgnoredApp = ref('')
const userLoggedIn = ref(false)
const userEmail = ref('user@example.com')

const autostart = ref(false)
const loading = ref(false)

// 添加快捷键设置所需的变量
const errorMsg = ref('')
const successMsg = ref('')
const currentShortcut = ref('')
let timer = null
const recordingShortcutType = ref('') // 当前正在录制的快捷键类型
const isRecording = ref(false) // 是否正在录制
let currentKeys = new Set() // 记录当前按下的键

// 导航项
const navItems = ref([
  { id: 'general', name: '通用设置', icon: Cog6ToothIcon },
  { id: 'shortcuts', name: '快捷键设置', icon: TvIcon },
  { id: 'clipboard', name: '剪贴板参数设置', icon: ClipboardIcon },
  { id: 'cloud', name: '云端入口', icon: CloudIcon },
  { id: 'user', name: '用户信息', icon: UserIcon }
])

// 设置数据
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
    toggleWindow: '',
    pasteWindow: '',
    quickPaste: '',
    clearHistory: ''
  }
})

// 用户信息
const userInfo = reactive({
  username: '当前用户',
  email: 'user@example.com',
  bio: '剪贴板管理爱好者'
})

// 方法定义
const setActiveNav = (navId) => {
  activeNav.value = navId
}

const goBack = () => {
  router.back()
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

const resetUserInfo = () => {
  Object.assign(userInfo, {
    username: '当前用户',
    email: 'user@example.com',
    bio: '剪贴板管理爱好者'
  })
  showMessage('用户信息已重置')
}


const showMessage = (message) => {
  toastMessage.value = message
  showToast.value = true
  setTimeout(() => {
    showToast.value = false
  }, 2000)
}

// 加载当前快捷键设置
const loadCurrentShortcuts = async () => {
  try {
    const toggleWindowShortcut = await invoke('get_current_shortcut')
    const pasteWindowShortcut = await invoke('get_current_shortcut2')
    
    settings.shortcuts.toggleWindow = toggleWindowShortcut || 'Shift+D'
    settings.shortcuts.pasteWindow = pasteWindowShortcut || 'Alt+Shift+C'
    
    console.log('加载当前快捷键:', {
      toggleWindow: settings.shortcuts.toggleWindow,
      pasteWindow: settings.shortcuts.pasteWindow
    })
  } catch (error) {
    console.error('加载快捷键失败:', error)
    // 设置默认值
    settings.shortcuts.toggleWindow = 'Shift+D'
    settings.shortcuts.pasteWindow = 'Alt+Shift+C'
  }
}

// 生命周期
onMounted(async () => {
  // 加载保存的设置
  const savedSettings = localStorage.getItem('clipboardSettings')
  if (savedSettings) {
    Object.assign(settings, JSON.parse(savedSettings))
  }
  await checkAutostartStatus()
  await loadCurrentShortcuts()
})

// 新添加
// 通用设置相关函数
// 启动时自动运行
// 检查自启状态
const checkAutostartStatus = async () => {
  try {
    const isEnabled = await invoke('is_autostart_enabled')
    settings.autoStart = isEnabled
    console.log('当前自启状态:', isEnabled)
  } catch (error) {
    console.error('检查自启状态失败:', error)
    showMessage('检查自启状态失败')
  }
}

// 切换自启状态 - 唯一的函数
const toggleAutoStart = async () => {
  loading.value = true
  try {
    await invoke('set_autostart', { enable: settings.autoStart })
    const message = settings.autoStart ? '已开启开机自启' : '已关闭开机自启'
    console.log(message)
    showMessage(message)
  } catch (error) {
    console.error('设置自启失败:', error)
    showMessage(`设置失败: ${error}`)
    // 出错时恢复原状态
    settings.autoStart = !settings.autoStart
  } finally {
    loading.value = false
  }
}
// 显示系统托盘图标
const toggleTrayIcon = async () => {
  try {
    await invoke('set_tray_icon_visibility', { visible: settings.showTrayIcon })
    showMessage(settings.showTrayIcon ? '已显示托盘图标' : '已隐藏托盘图标')
  } catch (error) {
    console.error('设置托盘图标失败:', error)
    settings.showTrayIcon = !settings.showTrayIcon
    showMessage(`设置失败: ${error}`)
  }
}

// 自动保存剪贴板历史
const toggleAutoSave = async () => {
  try {
    await invoke('set_auto_save', { enabled: settings.autoSave })
    showMessage(settings.autoSave ? '已启用自动保存' : '已禁用自动保存')
  } catch (error) {
    console.error('设置自动保存失败:', error)
    settings.autoSave = !settings.autoSave
    showMessage(`设置失败: ${error}`)
  }
}

// 历史记录保留时间
const updateRetentionDays = async () => {
  try {
    await invoke('set_retention_days', { days: parseInt(settings.retentionDays) })
    showMessage(`历史记录保留时间已设置为 ${settings.retentionDays} 天`)
  } catch (error) {
    console.error('设置保留时间失败:', error)
    showMessage(`设置失败: ${error}`)
  }
}

// 快捷键设置相关函数
// 开始录制快捷键
const startRecording = (shortcutType) => {
  recordingShortcutType.value = shortcutType
  isRecording.value = true
  currentKeys.clear()
  showMessage(`请按下 ${getShortcutDisplayName(shortcutType)} 的快捷键...`)
  
  // 添加全局键盘事件监听
  window.addEventListener('keydown', handleKeyDownDuringRecording)
  window.addEventListener('keyup', handleKeyUpDuringRecording)
}

const startRecording2 = (shortcutType) => {
  recordingShortcutType.value = shortcutType
  isRecording.value = true
  currentKeys.clear()
  showMessage(`请按下 ${getShortcutDisplayName(shortcutType)} 的快捷键...`)
  
  // 添加全局键盘事件监听
  window.addEventListener('keydown', handleKeyDownDuringRecording2)
  window.addEventListener('keyup', handleKeyUpDuringRecording)
}

// 处理录制期间的按键
const handleKeyDownDuringRecording = (event) => {
  if (!isRecording.value) return
  
  event.preventDefault()
  event.stopPropagation()
  
  // 记录按下的键
  const key = getKeyName(event)
  if (key) {
    currentKeys.add(key)
  }
  
  // 如果按下了 Escape 键，取消录制
  if (event.key === 'Escape') {
    cancelRecording()
    return
  }
  
  // 当有至少一个普通键（非修饰键）被按下时，完成录制
  const hasRegularKey = Array.from(currentKeys).some(key => 
    !['Ctrl', 'Alt', 'Shift', 'Meta'].includes(key)
  )
  
  if (hasRegularKey && currentKeys.size > 0) {
    const shortcutStr = Array.from(currentKeys).join('+')
    finishRecording(shortcutStr)
  }
}

const handleKeyDownDuringRecording2 = (event) => {
  if (!isRecording.value) return
  
  event.preventDefault()
  event.stopPropagation()
  
  // 记录按下的键
  const key = getKeyName(event)
  if (key) {
    currentKeys.add(key)
  }
  
  // 如果按下了 Escape 键，取消录制
  if (event.key === 'Escape') {
    cancelRecording()
    return
  }
  
  // 当有至少一个普通键（非修饰键）被按下时，完成录制
  const hasRegularKey = Array.from(currentKeys).some(key => 
    !['Ctrl', 'Alt', 'Shift', 'Meta'].includes(key)
  )
  
  if (hasRegularKey && currentKeys.size > 0) {
    const shortcutStr = Array.from(currentKeys).join('+')
    finishRecording2(shortcutStr)
  }
}

// 处理按键释放
const handleKeyUpDuringRecording = (event) => {
  if (!isRecording.value) return
  
  const key = getKeyName(event)
  if (key) {
    currentKeys.delete(key)
  }
}

// 获取按键名称
const getKeyName = (event) => {
  if (event.key === 'Control') return 'Ctrl'
  if (event.key === 'Alt') return 'Alt'
  if (event.key === 'Shift') return 'Shift'
  if (event.key === 'Meta') return 'Meta'
  
  // 排除修饰键
  if (event.key === 'Control' || event.key === 'Alt' || 
      event.key === 'Shift' || event.key === 'Meta') {
    return null
  }
  
  // 处理特殊按键
  if (event.key === ' ') return 'Space'
  if (event.key === 'Escape') return 'Escape'
  
  // 处理功能键
  if (event.key.startsWith('F') && event.key.length > 1) {
    const fNumber = event.key.slice(1)
    if (!isNaN(fNumber)) {
      return event.key
    }
  }
  
  // 处理字母键（转换为大写）
  if (event.key.length === 1 && event.key.match(/[a-zA-Z]/)) {
    return event.key.toUpperCase()
  }
  
  // 处理数字键
  if (event.key.match(/^[0-9]$/)) {
    return event.key
  }
  
  // 处理其他常见按键
  const specialKeys = {
    'ArrowUp': 'Up',
    'ArrowDown': 'Down', 
    'ArrowLeft': 'Left',
    'ArrowRight': 'Right',
    'Enter': 'Enter',
    'Tab': 'Tab',
    'CapsLock': 'CapsLock',
    'Backspace': 'Backspace',
    'Delete': 'Delete',
    'Insert': 'Insert',
    'Home': 'Home',
    'End': 'End',
    'PageUp': 'PageUp',
    'PageDown': 'PageDown',
    ' ': 'Space'
  }
  
  return specialKeys[event.key] || event.key
}

// 完成录制并设置快捷键
const finishRecording = async (newShortcut) => {
  isRecording.value = false
  const shortcutType = recordingShortcutType.value
  recordingShortcutType.value = ''
  
  // 移除事件监听
  window.removeEventListener('keydown', handleKeyDownDuringRecording)
  window.removeEventListener('keyup', handleKeyUpDuringRecording)
  
  // 调用你的 setShortcut 函数
  await setShortcut(newShortcut, shortcutType)
}

const finishRecording2 = async (newShortcut) => {
  isRecording.value = false
  const shortcutType = recordingShortcutType.value
  recordingShortcutType.value = ''
  
  // 移除事件监听
  window.removeEventListener('keydown', handleKeyDownDuringRecording)
  window.removeEventListener('keyup', handleKeyUpDuringRecording)
  
  // 调用你的 setShortcut 函数
  await setShortcut2(newShortcut, shortcutType)
}

const setShortcut = async (newShortcutStr, shortcutType = null) => {
  const targetType = shortcutType || recordingShortcutType.value
  if (!targetType) {
    console.error('没有指定快捷键类型')
    return
  }
  
  errorMsg.value = '';
  successMsg.value = '';

  try {
    // 根据后端函数，只传递 new_shortcut_str 参数
    await invoke('update_shortcut', { 
      newShortcutStr: newShortcutStr 
    });

    // 更新界面显示
    settings.shortcuts[targetType] = newShortcutStr;
    successMsg.value = `${getShortcutDisplayName(targetType)} 快捷键设置成功！`;
    console.log(`✅ ${getShortcutDisplayName(targetType)} 快捷键已成功更新为: ${newShortcutStr}`);

    await loadCurrentShortcuts();
  } catch (err) {
    errorMsg.value = `设置失败: ${err}`;
    console.error('❌ 设置快捷键失败:', err);
    
    // 如果出错，可能是因为快捷键冲突，提示用户
    if (err.includes('Failed to unregister hotkey') || err.includes('GlobalHotkey') || err.includes('可能已被占用')) {
      errorMsg.value = '快捷键设置失败：可能与其他程序冲突，请尝试其他组合键';
    }
  }

  // 3秒后自动清除提示消息
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    successMsg.value = '';
    errorMsg.value = '';
  }, 3000);
}

const setShortcut2 = async (newShortcutStr, shortcutType = null) => {
  const targetType = shortcutType || recordingShortcutType.value
  if (!targetType) {
    console.error('没有指定快捷键类型')
    return
  }
  
  errorMsg.value = '';
  successMsg.value = '';

  try {
    // 根据后端函数，只传递 new_shortcut_str 参数
    await invoke('update_shortcut2', { 
      newShortcutStr: newShortcutStr 
    });

    // 更新界面显示
    settings.shortcuts[targetType] = newShortcutStr;
    successMsg.value = `${getShortcutDisplayName(targetType)} 快捷键设置成功！`;
    console.log(`✅ ${getShortcutDisplayName(targetType)} 快捷键已成功更新为: ${newShortcutStr}`);
    await loadCurrentShortcuts();
  } catch (err) {
    errorMsg.value = `设置失败: ${err}`;
    console.error('❌ 设置快捷键失败:', err);
    
    // 如果出错，可能是因为快捷键冲突，提示用户
    if (err.includes('Failed to unregister hotkey') || err.includes('GlobalHotkey') || err.includes('可能已被占用')) {
      errorMsg.value = '快捷键设置失败：可能与其他程序冲突，请尝试其他组合键';
    }
  }

  // 3秒后自动清除提示消息
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => {
    successMsg.value = '';
    errorMsg.value = '';
  }, 3000);
}

// 辅助函数：获取快捷键显示名称
const getShortcutDisplayName = (shortcutType) => {
  const nameMap = {
    'toggleWindow': '显示/隐藏主窗口',
    'asteWindow': '显示/隐藏剪贴板',
    'quickPaste': '快速粘贴', 
    'clearHistory': '清空剪贴板历史'
  };
  return nameMap[shortcutType] || shortcutType;
}

// 取消录制（可选）
const cancelRecording = () => {
  isRecording.value = false
  recordingShortcutType.value = ''
  window.removeEventListener('keydown', handleKeyDownDuringRecording)
  window.removeEventListener('keyup', handleKeyUpDuringRecording)
  showMessage('已取消快捷键设置')
}

// 剪贴板参数设置相关函数
// 最大历史记录数量
const updateMaxHistoryItems = async () => {
  try {
    await invoke('set_max_history_items', { maxItems: settings.maxHistoryItems })
    showMessage(`最大历史记录数量已设置为 ${settings.maxHistoryItems}`)
  } catch (error) {
    console.error('设置最大历史记录数量失败:', error)
    showMessage(`设置失败: ${error}`)
  }
}

// 忽略短文本
const updateIgnoreShortText = async () => {
  try {
    await invoke('set_ignore_short_text', { minLength: settings.ignoreShortText })
    showMessage(`已设置忽略 ${settings.ignoreShortText} 字符以下的文本`)
  } catch (error) {
    console.error('设置忽略短文本失败:', error)
    showMessage(`设置失败: ${error}`)
  }
}

// 添加忽略应用
const addIgnoredApp = async () => {
  if (newIgnoredApp.value.trim() && !settings.ignoredApps.includes(newIgnoredApp.value.trim())) {
    const newApp = newIgnoredApp.value.trim()
    settings.ignoredApps.push(newApp)
    newIgnoredApp.value = ''
    
    try {
      await invoke('add_ignored_app', { appName: newApp })
      showMessage(`已添加忽略应用: ${newApp}`)
    } catch (error) {
      console.error('添加忽略应用失败:', error)
      settings.ignoredApps.pop() // 回滚
      showMessage(`添加失败: ${error}`)
    }
  }
}

// 移除忽略应用
const removeIgnoredApp = async (index) => {
  const removedApp = settings.ignoredApps[index]
  settings.ignoredApps.splice(index, 1)
  
  try {
    await invoke('remove_ignored_app', { appName: removedApp })
    showMessage(`已移除忽略应用: ${removedApp}`)
  } catch (error) {
    console.error('移除忽略应用失败:', error)
    settings.ignoredApps.splice(index, 0, removedApp) // 回滚
    showMessage(`移除失败: ${error}`)
  }
}

// 文本预览长度
const updatePreviewLength = async () => {
  try {
    await invoke('set_preview_length', { length: settings.previewLength })
    showMessage(`文本预览长度已设置为 ${settings.previewLength} 字符`)
  } catch (error) {
    console.error('设置预览长度失败:', error)
    showMessage(`设置失败: ${error}`)
  }
}

// 清空所有忽略应用
const clearAllIgnoredApps = async () => {
  if (settings.ignoredApps.length === 0) {
    showMessage('没有可清空的忽略应用')
    return
  }
  
  if (confirm('确定要清空所有忽略应用吗？')) {
    const oldApps = [...settings.ignoredApps]
    settings.ignoredApps = []
    
    try {
      await invoke('clear_all_ignored_apps')
      showMessage('已清空所有忽略应用')
    } catch (error) {
      console.error('清空忽略应用失败:', error)
      settings.ignoredApps = oldApps // 回滚
      showMessage(`清空失败: ${error}`)
    }
  }
}

// 云端同步相关函数
// 启用/禁用云端同步
const toggleCloudSync = async () => {
  try {
    await invoke('set_cloud_sync', { enabled: settings.cloudSync })
    showMessage(settings.cloudSync ? '已启用云端同步' : '已禁用云端同步')
  } catch (error) {
    console.error('设置云端同步失败:', error)
    settings.cloudSync = !settings.cloudSync
    showMessage(`设置失败: ${error}`)
  }
}

// 同步频率
const updateSyncFrequency = async () => {
  try {
    await invoke('set_sync_frequency', { frequency: settings.syncFrequency })
    showMessage(`同步频率已设置为 ${getFrequencyText(settings.syncFrequency)}`)
  } catch (error) {
    console.error('设置同步频率失败:', error)
    showMessage(`设置失败: ${error}`)
  }
}

// 加密同步数据
const toggleEncryptCloudData = async () => {
  try {
    await invoke('set_encrypt_cloud_data', { enabled: settings.encryptCloudData })
    showMessage(settings.encryptCloudData ? '已启用数据加密' : '已禁用数据加密')
  } catch (error) {
    console.error('设置数据加密失败:', error)
    settings.encryptCloudData = !settings.encryptCloudData
    showMessage(`设置失败: ${error}`)
  }
}

// 立即同步
const syncNow = async () => {
  try {
    showMessage('正在同步...')
    await invoke('force_cloud_sync')
    showMessage('云端同步完成')
  } catch (error) {
    console.error('同步失败:', error)
    showMessage(`同步失败: ${error}`)
  }
}

// 查看同步状态
const checkSyncStatus = async () => {
  try {
    const status = await invoke('get_sync_status')
    showMessage(`同步状态: ${status.lastSync ? `最后同步 ${formatTime(status.lastSync)}` : '从未同步'}`)
  } catch (error) {
    console.error('获取同步状态失败:', error)
    showMessage(`获取状态失败: ${error}`)
  }
}

// 辅助函数：获取频率文本
const getFrequencyText = (frequency) => {
  const frequencyMap = {
    'realtime': '实时',
    '5min': '5分钟',
    '15min': '15分钟', 
    '1hour': '1小时'
  }
  return frequencyMap[frequency] || frequency
}

// 用户信息相关函数
// 保存用户信息
const saveUserInfo = async () => {
  try {
    await invoke('update_user_profile', {
      username: userInfo.username,
      email: userInfo.email,
      bio: userInfo.bio
    })
    showMessage('用户信息已保存')
  } catch (error) {
    console.error('保存用户信息失败:', error)
    showMessage(`保存失败: ${error}`)
  }
}

// 更换头像
const changeAvatar = async () => {
  try {
    const filePath = await invoke('select_avatar_file')
    if (filePath) {
      await invoke('upload_user_avatar', { filePath })
      showMessage('头像更换成功')
    }
  } catch (error) {
    console.error('更换头像失败:', error)
    showMessage(`更换失败: ${error}`)
  }
}

// 修改密码
const changePassword = async () => {
  try {
    // 这里应该打开密码修改模态框
    const result = await invoke('open_change_password_dialog')
    if (result.success) {
      showMessage('密码修改成功')
    }
  } catch (error) {
    console.error('修改密码失败:', error)
    showMessage(`修改失败: ${error}`)
  }
}

// 删除账户
const deleteAccount = async () => {
  if (confirm('确定要删除账户吗？此操作将永久删除所有数据且不可恢复！')) {
    try {
      await invoke('delete_user_account')
      showMessage('账户已删除')
      router.push('/')
    } catch (error) {
      console.error('删除账户失败:', error)
      showMessage(`删除失败: ${error}`)
    }
  }
}

// 导出用户数据
const exportUserData = async () => {
  try {
    const exportPath = await invoke('export_user_data')
    showMessage(`用户数据已导出到: ${exportPath}`)
  } catch (error) {
    console.error('导出数据失败:', error)
    showMessage(`导出失败: ${error}`)
  }
}

// 导入用户数据
const importUserData = async () => {
  try {
    const importPath = await invoke('import_user_data')
    showMessage('用户数据导入成功')
    // 重新加载用户信息
    await loadUserInfo()
  } catch (error) {
    console.error('导入数据失败:', error)
    showMessage(`导入失败: ${error}`)
  }
}

// 加载用户信息
const loadUserInfo = async () => {
  try {
    const profile = await invoke('get_user_profile')
    Object.assign(userInfo, profile)
  } catch (error) {
    console.error('加载用户信息失败:', error)
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
  gap: 8px;
}

.nav-item:hover {
  background: #f1f3f5;
}

.nav-item.active {
  background: #e4edfd;
  color: #416afe;
}

.nav-icon {
  width: 1.2rem;
  height: 1.2rem;
  position: relative;
  top: 1px; 
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