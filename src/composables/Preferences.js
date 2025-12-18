import { ref, reactive, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { emit } from '@tauri-apps/api/event'
import { apiService,ensureAbsoluteAvatarUrl } from '../services/api'
import { useSettingsStore } from '../stores/settings'
import { loadUsername } from './Menu'
import { 
  Cog6ToothIcon,
  TvIcon,
  CloudIcon,
  ClipboardIcon,
  UserIcon
} from '@heroicons/vue/24/outline'
import { togglePrivateWindow } from '../utils/actions.js'

export function usePreferences() {
  const router = useRouter()
  const currentWindow = getCurrentWindow();

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

  // 注册相关状态
  const showRegisterDialog = ref(false)
  const showLoginDialog = ref(false)
  const registerLoading = ref(false)
  const loginLoading = ref(false)

  // 修改密码相关状态
  const showChangePasswordDialog = ref(false)
  const changePasswordLoading = ref(false)

  // 窗口关闭监听器
  let firstCloseWindow = true
  let unlistenCloseRequested = null
  
  // 注册表单数据
  const registerData = reactive({
    username: '',
    email: '',
    password: '',
    password2: ''
  })
  
  // 登录表单数据
  const loginData = reactive({
    username: '',
    password: ''
  })

  // 修改密码表单数据
  const changePasswordData = reactive({
    old_password: '',
    new_password: '',
    new_password2: '' 
  })
  
  // 表单验证错误
  const registerErrors = reactive({
    username: '',
    email: '',
    password: '',
    password2: ''
  })

  // 修改密码表单验证错误
  const changePasswordErrors = reactive({
    old_password: '',
    new_password: '',
    new_password2: ''
  })

  // 快捷键设置所需的变量
  const errorMsg = ref('')
  const successMsg = ref('')
  const currentShortcut = ref('')
  let timer = null
  const shortcutManager = reactive({
    currentType: '',
    isRecording: false,
    currentKeys: new Set()
  })
  const recordingShortcutType = ref('')

  // 同步状态相关数据
  const lastSyncTime = ref(null)
  const lastSyncStatus = ref('')
  const isSyncing = ref(false)

  // 用户信息
  const userInfo = reactive({
    username: '',
    email: '',
    bio: '',
    avatar: ''
  })

  // 导航项
  const navItems = ref([
    { id: 'general', name: '通用设置', icon: Cog6ToothIcon },
    { id: 'shortcuts', name: '快捷键设置', icon: TvIcon },
    { id: 'clipboard', name: '剪贴板参数设置', icon: ClipboardIcon },
    { id: 'ocr', name: 'OCR设置', icon: ClipboardIcon },
    { id: 'ai', name: 'AI Agent 设置', icon: ClipboardIcon },
    { id: 'security', name: '安全与隐私', icon: ClipboardIcon }, 
    { id: 'backup', name: '数据备份', icon: ClipboardIcon },
    { id: 'cloud', name: '云端入口', icon: CloudIcon },
    { id: 'user', name: '用户信息', icon: UserIcon }
  ])

  // 设置数据
  const settings = useSettingsStore().settings

  // 快捷键显示名称映射
  const shortcutDisplayNames = {
    global_shortcut: '显示/隐藏主窗口',
    global_shortcut_2: '显示/隐藏剪贴板', 
    global_shortcut_3: '显示/隐藏AI助手',
    global_shortcut_4: '显示/隐藏设置页面',
    global_shortcut_5: '清空剪贴板历史'
  }
  const shortcutKeys = Object.keys(shortcutDisplayNames)

  // 基础方法
  const setActiveNav = (navId) => {
    activeNav.value = navId
  }

  const goBack = () => {
    router.back()
  }

  // 查看隐私函数
  const showPrivate = () => {
    togglePrivateWindow()
  }

  // 表单验证函数
  const validateRegisterForm = () => {
    let isValid = true
    
    // 清除之前的错误
    Object.keys(registerErrors).forEach(key => {
      registerErrors[key] = ''
    })
    
    // 验证用户名
    if (!registerData.username.trim()) {
      registerErrors.username = '用户名不能为空'
      isValid = false
    } else if (registerData.username.length < 3) {
      registerErrors.username = '用户名至少3个字符'
      isValid = false
    }
    
    // 验证邮箱
    const emailRegex = /^[^\s@]+@[^\s@]+\.[^\s@]+$/
    if (!registerData.email.trim()) {
      registerErrors.email = '邮箱不能为空'
      isValid = false
    } else if (!emailRegex.test(registerData.email)) {
      registerErrors.email = '邮箱格式不正确'
      isValid = false
    }
    
    // 验证密码
    if (!registerData.password) {
      registerErrors.password = '密码不能为空'
      isValid = false
    } else if (registerData.password.length < 6) {
      registerErrors.password = '密码至少6个字符'
      isValid = false
    }
    
    // 验证确认密码
    if (!registerData.password2) {
      registerErrors.password2 = '请确认密码'
      isValid = false
    } else if (registerData.password !== registerData.password2) {
      registerErrors.password2 = '两次输入的密码不一致'
      isValid = false
    }
    
    return isValid
  }

  // 验证修改密码表单
  const validateChangePasswordForm = () => {
    let isValid = true
    
    // 清除之前的错误
    Object.keys(changePasswordErrors).forEach(key => {
      changePasswordErrors[key] = ''
    })
    
    // 验证旧密码
    if (!changePasswordData.old_password) {
      changePasswordErrors.old_password = '旧密码不能为空'
      isValid = false
    }
    
    // 验证新密码
    if (!changePasswordData.new_password) {
      changePasswordErrors.new_password = '新密码不能为空'
      isValid = false
    } else if (changePasswordData.new_password.length < 6) {
      changePasswordErrors.new_password = '新密码至少6个字符'
      isValid = false
    }
    
    // 验证确认新密码
    if (!changePasswordData.new_password2) {
      changePasswordErrors.new_password2 = '请确认新密码'
      isValid = false
    } else if (changePasswordData.new_password !== changePasswordData.new_password2) {
      changePasswordErrors.new_password2 = '两次输入的新密码不一致'
      isValid = false
    }
    
    return isValid
  }

  // 注册方法
  const handleRegister = async () => {
    // 验证表单
    if (!validateRegisterForm()) {
      showMessage('请填写正确的表单信息', 'error')
      return
    }
    
    registerLoading.value = true
    
    try {
      const response = await apiService.register({
        username: registerData.username,
        email: registerData.email,
        password: registerData.password,
        password2: registerData.password2
      })

      if (response.success) {
        // 注册成功
        showMessage('注册成功！', 'success')
        console.log('登录成功返回信息:', response.data)
        
        // 关闭注册对话框
        showRegisterDialog.value = false

        const responselogin = await apiService.login({
        username: registerData.username,
        password: registerData.password
        })

        if (responselogin.success) {
          // 登录成功
          showMessage('登录成功！', 'success')
          console.log('登录成功返回信息:', responselogin.data)
          // 保存用户信息到本地存储
          if (responselogin.data) {
            localStorage.setItem('user', JSON.stringify(responselogin.data))
            userLoggedIn.value = true
            userEmail.value = responselogin.data.user.email || loginData.email
            userInfo.username = responselogin.data.user.username || '当前用户'
            userInfo.email = responselogin.data.user.email || loginData.email
            userInfo.bio = responselogin.data.user.bio
            userInfo.avatar = responselogin.data.user.avatar || ''
          }
          loadUsername()
        }

        // 清空表单数据
        Object.assign(registerData, {
          username: '',
          email: '',
          password: '',
          password2: ''
        })
        
        // 清除错误信息
        Object.keys(registerErrors).forEach(key => {
          registerErrors[key] = ''
        })
      } else {
        // 注册失败
        let errorMessage = '注册失败'
        
        if (response.data && typeof response.data === 'object') {
          // 创建更易读的错误信息
          const errorLines = []
          
          for (const [field, errors] of Object.entries(response.data)) {
            if (Array.isArray(errors)) {
              // 将字段名转换为中文
              const fieldName = field === 'email' ? '邮箱' : 
                              field === 'password' ? '密码' : 
                              field === 'username' ? '用户名' : field
              
              // 处理每个错误项
              errors.forEach(error => {
                errorLines.push(`• ${fieldName}: ${error}`)
              })
            }
          }
          
          if (errorLines.length > 0) {
            // 分行显示，更清晰
            errorMessage = `注册失败：\n${errorLines.join('\n')}`
          }
        }
        
        showMessage(errorMessage)
        console.error('注册失败返回信息:', response.data)
      }
    } catch (error) {
      console.error('注册错误:', error)
      showMessage('注册出错，请稍后重试', 'error')
    } finally {
      registerLoading.value = false
    }
  }

  // 登录方法
  const handleLogin = async () => {
    if (!loginData.username || !loginData.password) {
      showMessage('请输入用户名和密码', 'error')
      return
    }
    
    loginLoading.value = true
    
    try {
      // 这里调用登录API
      const response = await apiService.login({
        username: loginData.username,
        password: loginData.password
      })

      if (response.success) {
        // 登录成功
        showMessage('登录成功！', 'success')
        console.log('登录成功返回信息:', response.data)
        // 保存用户信息到本地存储
        if (response.data) {
          localStorage.setItem('user', JSON.stringify(response.data))
          userLoggedIn.value = true
          userEmail.value = response.data.user.email || loginData.email
          userInfo.username = response.data.user.username || '当前用户'
          userInfo.email = response.data.user.email || loginData.email
          userInfo.bio = response.data.user.bio
          userInfo.avatar = response.data.user.avatar || ''
        }
        loadUsername()
        // 关闭登录对话框
        showLoginDialog.value = false
        await handleCloudPull(true);
        
        // 清空表单数据
        Object.assign(loginData, {
          username: '',
          password: ''
        })
      } else {
        // 登录失败
        showMessage(`登录失败：${response.message}`, 'error')
        console.error('登录失败返回信息:', response.data)
      }
    } catch (error) {
      console.error('登录错误:', error)
      showMessage('登录出错，请检查网络连接', 'error')
    } finally {
      loginLoading.value = false
    }
  }

  // 打开注册对话框
  const openRegisterDialog = () => {
    showRegisterDialog.value = true
    // 清空表单数据
    Object.assign(registerData, {
      username: '',
      email: '',
      password: '',
      password2: ''
    })
    // 清空错误信息
    Object.keys(registerErrors).forEach(key => {
      registerErrors[key] = ''
    })
  }

  // 打开登录对话框
  const openLoginDialog = () => {
    showLoginDialog.value = true
  }

  // 关闭注册对话框
  const closeRegisterDialog = () => {
    showRegisterDialog.value = false
  }

  // 关闭登录对话框
  const closeLoginDialog = () => {
    showLoginDialog.value = false
  }

  // 打开修改密码对话框
  const openChangePasswordDialog = () => {
    if (!userLoggedIn.value) {
      showMessage('请先登录才能修改密码', 'warning')
      return
    }
    showChangePasswordDialog.value = true
    // 清空表单数据
    Object.assign(changePasswordData, {
      old_password: '',
      new_password: '',
      new_password2: ''
    })
    // 清空错误信息
    Object.keys(changePasswordErrors).forEach(key => {
      changePasswordErrors[key] = ''
    })
  }
  
  // 关闭修改密码对话框
  const closeChangePasswordDialog = () => {
    showChangePasswordDialog.value = false
  }

  const login = () => {
    openLoginDialog()
  }

  // 修改logout方法
  const logout = async () => {
    const message = '确定要退出登录吗？';
    const confirmed = await window.confirm(message);
    if (confirmed) {
      localStorage.removeItem('user')
      localStorage.removeItem('token')
      userLoggedIn.value = false
      userEmail.value = ''
      Object.assign(userInfo, {
        username: '',
        email: '',
        bio: '',
        avatar: ''
      })
      showMessage('已退出登录', 'success')
    }
  }

  // 更新本地存储中的用户信息
  const updateUserInfo = async () => {
    try {
      const apiResponse = await apiService.updateProfile({
        bio: userInfo.bio
      });

      if (!apiResponse.success) {
        // API调用失败，显示错误信息
        showMessage(apiResponse.message || '更新个人简介失败', 'error');
        console.error('更新个人简介失败返回信息:', apiResponse.data);
        return; 
      } 
      
      const savedUserJson = localStorage.getItem('user')
       if (savedUserJson) {
         let userData = JSON.parse(savedUserJson)
         
         // 确保结构存在，并更新 user.bio 字段
         if (userData) {
           userData.user.bio = userInfo.bio
           localStorage.setItem('user', JSON.stringify(userData))
           showMessage('个人简介已保存', 'success')
         } else {
           console.error('localStorage 中的 user 数据结构不正确或缺失 user.user 属性')
         }
       }
     } catch (error) {
       console.error('保存个人简介到 localStorage 失败:', error)
     }
  };

  const resetUserInfo = () => {
    Object.assign(userInfo, {
      username: '当前用户',
      email: 'user@example.com',
      bio: '剪贴板管理爱好者',
      avatar: ''
    })
    showMessage('用户信息已重置')
  }

  const showMessage = (message, type = 'success') => {
    toastMessage.value = message
    showToast.value = true
    setTimeout(() => {
      showToast.value = false
    }, 2000)
  }


  // 通用设置相关函数
// 启动时自动运行
// 检查自启状态
/*
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

//启动时最小化到托盘
const toggleMinimizeToTray = async () => {
  try {
    await invoke('set_minimize_to_tray', { enabled: settings.showTrayIcon })
    showMessage(settings.showTrayIcon ? '已启用启动时最小化到托盘' : '已禁用启动时最小化到托盘')
  } catch (error) {
    console.error('设置最小化到托盘失败:', error)
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
}*/

  // 快捷键相关方法
  const startRecording = (shortcutType) => {
    shortcutManager.currentType = shortcutType
    shortcutManager.isRecording = true
    shortcutManager.currentKeys.clear()
    
    showMessage(`请按下 ${shortcutDisplayNames[shortcutType]} 的快捷键...`)
    
    window.addEventListener('keydown', handleKeyDownDuringRecording)
    window.addEventListener('keyup', handleKeyUpDuringRecording)
  }

  const handleKeyDownDuringRecording = (event) => {
    if (!shortcutManager.isRecording) return
    
    event.preventDefault()
    event.stopPropagation()
    
    const key = getKeyName(event)
    if (key) {
      shortcutManager.currentKeys.add(key)
    }
    
    if (event.key === 'Escape') {
      cancelRecording()
      return
    }
    
    const hasRegularKey = Array.from(shortcutManager.currentKeys).some(key => 
      !['Ctrl', 'Alt', 'Shift', 'Meta'].includes(key)
    )
    
    if (hasRegularKey && shortcutManager.currentKeys.size > 0) {
      const shortcutStr = Array.from(shortcutManager.currentKeys).join('+')
      finishRecording(shortcutStr)
    }
  }

  const handleKeyUpDuringRecording = (event) => {
    if (!shortcutManager.isRecording) return
    
    const key = getKeyName(event)
    if (key) {
      shortcutManager.currentKeys.delete(key)
    }
  }

  const getKeyName = (event) => {
    if (event.key === 'Control') return 'Ctrl'
    if (event.key === 'Alt') return 'Alt'
    if (event.key === 'Shift') return 'Shift'
    if (event.key === 'Meta') return 'Meta'
    
    if (event.key === 'Control' || event.key === 'Alt' || 
        event.key === 'Shift' || event.key === 'Meta') {
      return null
    }
    
    if (event.key === ' ') return 'Space'
    if (event.key === 'Escape') return 'Escape'
    
    if (event.key.startsWith('F') && event.key.length > 1) {
      const fNumber = event.key.slice(1)
      if (!isNaN(fNumber)) {
        return event.key
      }
    }
    
    if (event.key.length === 1 && event.key.match(/[a-zA-Z]/)) {
      return event.key.toUpperCase()
    }
    
    if (event.key.match(/^[0-9]$/)) {
      return event.key
    }
    
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

  const finishRecording = async (newShortcut) => {
    shortcutManager.isRecording = false
    
    window.removeEventListener('keydown', handleKeyDownDuringRecording)
    window.removeEventListener('keyup', handleKeyUpDuringRecording)
    
    await setShortcut(newShortcut, shortcutManager.currentType)
    shortcutManager.currentType = ''
  }

  const setShortcut = async (newShortcutStr, shortcutType) => {
    if (!shortcutType) {
      console.error('没有指定快捷键类型')
      return
    }
    
    errorMsg.value = ''
    successMsg.value = ''

    try {
      await invoke('update_shortcut', { 
        shortcutType: shortcutType,
        newShortcutStr: newShortcutStr 
      })

      await updateSetting(shortcutType, newShortcutStr)
      successMsg.value = `${shortcutDisplayNames[shortcutType]} 快捷键设置成功！`
      console.log(`✅ ${shortcutDisplayNames[shortcutType]} 快捷键已更新为: ${newShortcutStr}`)

    } catch (err) {
      errorMsg.value = `设置失败: ${err}`
      console.error('❌ 设置快捷键失败:', err)
      
      if (err.includes('Failed to unregister hotkey') || err.includes('GlobalHotkey') || err.includes('可能已被占用')) {
        errorMsg.value = '快捷键设置失败：可能与其他程序冲突，请尝试其他组合键'
      }
    }

    if (timer) clearTimeout(timer)
    timer = setTimeout(() => {
      successMsg.value = ''
      errorMsg.value = ''
    }, 3000)
  }

  const cancelRecording = () => {
    shortcutManager.isRecording = false
    shortcutManager.currentType = ''
    window.removeEventListener('keydown', handleKeyDownDuringRecording)
    window.removeEventListener('keyup', handleKeyUpDuringRecording)
    showMessage('已取消快捷键设置')
  }

  // 设置相关方法
  const updateSetting = async (key, value) => {
    const oldValue = settings[key]
    
    try {
      settings[key] = value
      await invoke('set_config_item', { key, value })
      showMessage('设置已更新')

      // 如果更新的是 ai_enabled，发送事件到主窗口
      if (key === 'ai_enabled') {
        await emit('ai-enabled-changed', { 
          enabled: value 
        })
        console.log(`📡 发送 ai_enabled 变更事件: ${value}`)
      }
    } catch (error) {
      console.error(`设置 ${key} 失败:`, error)
      settings[key] = oldValue
      showMessage(`设置失败: ${error}`)
    }
  }

  const toggleOCRLanguage = async (language, isChecked) => {
    let updatedLanguages
    
    if (isChecked) {
      updatedLanguages = [...settings.ocr_languages, language]
    } else {
      updatedLanguages = settings.ocr_languages.filter(lang => lang !== language)
    }
    
    try {
      await updateSetting('ocr_languages', updatedLanguages)
      showMessage('OCR语言设置已更新')
    } catch (error) {
      console.error('更新OCR语言失败:', error)
      showMessage(`更新失败: ${error}`)
    }
  }

  const changeStoragePath = async () => {
    try {
      const selectedPath = await open({
        directory: true,
        multiple: false,
        title: '选择数据存储路径',
        defaultPath: settings.storage_path || undefined
      })

      if (selectedPath) {
        settings.storage_path = selectedPath
        await updateSetting('storage_path', selectedPath)
        showMessage('存储路径已更新')
      }
    } catch (error) {
      console.error('选择存储路径失败:', error)
      showMessage(`选择路径失败: ${error}`)
    }
  }

  // 数据管理方法
  const clearAiHistory = async () => {
    if (confirm('确定要清空所有AI对话历史吗？此操作不可恢复。')) {
      try {
        // await invoke('clear_ai_history')
        showMessage('AI对话历史已清空')
      } catch (error) {
        console.error('清空AI历史失败:', error)
        showMessage(`清空失败: ${error}`)
      }
    }
  }

  const exportData = async () => {
    try {
      await invoke('export_to_zip')
      showMessage(`数据已导出到: ${settings.storage_path}/SmartPaste_Backup.zip`)
    } catch (error) {
      console.error('导出数据失败:', error)
      showMessage(`导出失败: ${error}`)
    }
  }

  const importData = async () => {
    try {
      await invoke('import_data_from_zip')
      showMessage('数据导入成功')
    } catch (error) {
      console.error('导入数据失败:', error)
      showMessage(`导入失败: ${error}`)
    }
  }

  const createBackup = async () => {
    try {
      // const backupPath = await invoke('create_backup')
      showMessage(`备份已创建: ${backupPath}`)
    } catch (error) {
      console.error('创建备份失败:', error)
      showMessage(`备份失败: ${error}`)
    }
  }

  // 云端同步方法
  const formatTime = (timestamp) => {
    if (!timestamp) return ''
    const date = new Date(timestamp)
    return `${date.getFullYear()}-${(date.getMonth() + 1).toString().padStart(2, '0')}-${date.getDate().toString().padStart(2, '0')} ${date.getHours().toString().padStart(2, '0')}:${date.getMinutes().toString().padStart(2, '0')}`
  }

  const manualSync = async () => {
    if (isSyncing.value) return
    
    isSyncing.value = true
    try {
      // await invoke('force_cloud_sync')
      lastSyncStatus.value = 'success'
      lastSyncTime.value = Date.now()
      localStorage.setItem('lastSyncTime', lastSyncTime.value)
      showMessage('同步成功')
    } catch (error) {
      lastSyncStatus.value = 'error'
      console.error('同步失败:', error)
      showMessage(`同步失败: ${error}`)
    } finally {
      isSyncing.value = false
    }
  }

  const syncNow = async () => {
    try {
      showMessage('正在同步...')
      // await invoke('force_cloud_sync')
      showMessage('云端同步完成')
    } catch (error) {
      console.error('同步失败:', error)
      showMessage(`同步失败: ${error}`)
    }
  }

  const checkSyncStatus = async () => {
    try {
      // const status = await invoke('get_sync_status')
      showMessage(`同步状态: ${status.lastSync ? `最后同步 ${formatTime(status.lastSync)}` : '从未同步'}`)
    } catch (error) {
      console.error('获取同步状态失败:', error)
      showMessage(`获取状态失败: ${error}`)
    }
  }

  // 用户管理方法
  // 修改密码方法
  const handleChangePassword = async () => {
    if (!validateChangePasswordForm()) {
      showMessage('请填写正确的表单信息', 'error')
      return
    }
    
    if (!userLoggedIn.value) {
      showMessage('请先登录', 'error')
      return
    }

    // 1. 在这里获取 Refresh Token
    let refreshToken = null
    try {
      const userString = localStorage.getItem('user');
      if (userString) {
        const user = JSON.parse(userString);
        refreshToken = user.jwt.refresh;
      }
    } catch (e) {
      console.error('解析本地用户信息失败:', e);
    }
    
    if (!refreshToken) {
      showMessage('无法获取登录状态，请重新登录', 'error')
      return
    }

    changePasswordLoading.value = true
    
    try {
      // 2. 调用 API Service
      const response = await apiService.changePassword(
        changePasswordData, // 包含三个密码字段
        refreshToken      // 传入 refresh token
      )

      if (response.success) {
        showMessage('密码修改成功！请重新登录', 'success')
        
        // 强制退出登录并清空状态
        localStorage.removeItem('user')
        localStorage.removeItem('token')
        userLoggedIn.value = false
        userEmail.value = ''
        Object.assign(userInfo, { username: '', email: '', bio: '', avatar: '' })
        
        // 关闭对话框并清空表单
        showChangePasswordDialog.value = false
        Object.assign(changePasswordData, {
          old_password: '',
          new_password: '',
          new_password2: ''
        })
        Object.keys(changePasswordErrors).forEach(key => {
          changePasswordErrors[key] = ''
        })
        
        // 建议：可以添加页面跳转或刷新逻辑

      } else {
        // API 返回错误
        showMessage(`密码修改失败：${response.message}`, 'error')
      }
    } catch (error) {
      console.error('密码修改错误:', error)
      showMessage('密码修改出错，请检查网络连接', 'error')
    } finally {
      changePasswordLoading.value = false
    }
  }

  // 更换头像方法
  const changeAvatar = async () => {
    if (!userLoggedIn.value) {
      showMessage('请先登录才能更换头像', 'warning')
      return
    }

    try {
      // 打开文件选择对话框，只允许图片
      const selectedPath = await open({
        directory: false,
        multiple: false,
        title: '选择新头像文件',
        filters: [{
          name: 'Image',
          extensions: ['png', 'jpg', 'jpeg', 'webp']
        }]
      })

      if (!selectedPath) {
        return // 用户取消选择
      }
      
      // 获取文件信息
      const filePath = Array.isArray(selectedPath) ? selectedPath[0] : selectedPath
      console.log('获取的头像路径：', filePath)
      const fileName = filePath.substring(filePath.lastIndexOf('\\') + 1)
      const fileExtension = fileName.split('.').pop().toLowerCase()
      const mimeType = {
        'png': 'image/png',
        'jpg': 'image/jpeg',
        'jpeg': 'image/jpeg',
        'webp': 'image/webp'
      }[fileExtension] || 'application/octet-stream'

      if (mimeType === 'application/octet-stream') {
        showMessage('文件类型不支持，请选择 PNG/JPG/WEBP 格式', 'error')
        return
      }

      showMessage('正在读取并上传头像...')
      
      // 读取文件内容为 Base64 编码字符串
      // 该命令接收文件路径，读取文件内容并返回 Base64 编码字符串。
      let base64Content = null;
      try {
          base64Content = await invoke('read_file_base64', { filePath });
      } catch (e) {
          console.error('读取本地文件失败:', e);
          showMessage('读取本地文件失败，请确保 Rust 命令已实现', 'error');
          return;
      }
      
      // 将 Base64 转换为 File 对象
      // 移除可能的前缀 'data:mime/type;base64,'
      const base64Data = base64Content.split(',').pop();
      const binaryString = atob(base64Data);
      const len = binaryString.length;
      const bytes = new Uint8Array(len);
      for (let i = 0; i < len; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      // 创建 File 对象，供 fetch API 上传
      const fileObject = new File([bytes], fileName, { type: mimeType });

      // 调用 API Service 上传
      const apiResponse = await apiService.uploadAvatar(fileObject);

      if (apiResponse.success) {
        // 更新 UI 状态
        // apiService.uploadAvatar 中已更新 localstorage，这里同步到响应式状态
        const savedUser = localStorage.getItem('user');
        if (savedUser) {
            const userData = JSON.parse(savedUser);
            // 确保同步最新的 avatar URL
            userInfo.avatar = userData.user.avatar || userInfo.avatar; 
        }

        showMessage('头像更换成功', 'success');
      } else {
        showMessage(apiResponse.message || '头像上传失败', 'error');
      }
    } catch (error) {
      console.error('更换头像错误:', error);
      showMessage(`更换失败: ${error.message || '网络错误'}`, 'error');
    }
  }

  const deleteAccount = async () => {
    if (!userLoggedIn.value) {
      showMessage('请先登录才能删除账户', 'warning');
      return;
    }
    const message = '确定要删除账户吗？';
    const confirmed = await window.confirm(message);
    if (confirmed) {
      loading.value = true;
      let refreshToken = null
      try {
        const userString = localStorage.getItem('user');
        if (userString) {
          const user = JSON.parse(userString);
          refreshToken = user.jwt.refresh;
        }
      } catch (e) {
        console.error('解析本地用户信息失败:', e);
      }

      if (!refreshToken) {
        showMessage('无法获取登录状态，请重新登录', 'error')
        return
      }

      try {
        // 调用后端API删除账户
        const apiResponse = await apiService.deleteAccount(refreshToken);

        if (apiResponse.success) {
          // 清空本地登录状态
          localStorage.removeItem('user');
          localStorage.removeItem('token');
          userLoggedIn.value = false;
          userEmail.value = '';
          Object.assign(userInfo, { username: '', email: '', bio: '', avatar: '' });
          
          showMessage('账户已成功删除', 'success');
          // 删除成功后跳转到主页或登录页
          //router.push('/');
        } else {
          // API 调用失败
          showMessage(apiResponse.message || '删除账户失败', 'error');
          console.error('删除账户失败返回信息:', apiResponse.data);
        }
      } catch (error) {
        console.error('删除账户错误:', error);
        showMessage(`删除失败: ${error.message || '网络错误'}`, 'error');
      } finally {
        loading.value = false;
      }
    }
  }

  /**
   * 云端推送/上传主函数 (直接对接后端接口)
   */
  const handleCloudPush = async () => {
    if (isSyncing.value) return;
    
    // 权限预检查
    if (!localStorage.getItem('token')) {
      showMessage('请先登录后进行同步', 'error');
      return;
    }

    isSyncing.value = true;
    
    try {
      showMessage('正在推送数据至云端...', 'info');

      // 1. 同步配置：从本地读取并上传
      // 直接调用 Rust 获取文件内容，随后对接 Web 接口
      const configTxt = await invoke('get_config_json'); 
      const configRes = await apiService.uploadConfig(configTxt);
      if (!configRes.success) throw new Error(`配置同步失败: ${configRes.message}`);

      // 2. 同步数据库：读本地DB并推送
      // 后端 views.py 的 SqlitePushView 会自动合并数据
      const dbBase64 = await invoke('read_db_file_base64');
      const dbBlob = base64ToBlob(dbBase64, 'application/x-sqlite3');
      const dbRes = await apiService.pushSqliteDatabase(dbBlob);
      if (!dbRes.success) throw new Error(`数据库推送失败: ${dbRes.message}`);

      // 3. 同步文件：遍历本地目录并逐个上传
      const localFiles = await invoke('get_local_files_to_upload');
      for (const fileInfo of localFiles) {
        // 读取二进制内容并转为 Web 可发送的 Blob 对象
        const content = await invoke('read_file_base64', { filePath: fileInfo.file_path });
        const blob = base64ToBlob(content, 'application/octet-stream');
        
        // 调用 apiService 接口上传，后端会根据 relative_path 自动处理覆盖或新增
        const fileRes = await apiService.uploadClipboardFile(blob, fileInfo.relative_path);
        if (!fileRes.success) {
          console.warn(`文件上传失败 (${fileInfo.relative_path}):`, fileRes.message);
        }
      }

      // 成功处理
      showMessage('云端数据推送成功！', 'success');
      lastSyncTime.value = Date.now();
      localStorage.setItem('lastSyncTime', lastSyncTime.value);

    } catch (error) {
      // 错误处理逻辑：打印日志并反馈给用户
      console.error('云端推送错误:', error);
      showMessage(error.message || '网络同步出错，请检查连接', 'error');
    } finally {
      isSyncing.value = false;
    }
  };

  const handleCloudPull = async (isSilent = false) => {
    if (isSyncing.value) return;
    isSyncing.value = true;
    try {
      if (!isSilent) showMessage('正在同步云端数据...', 'info');

      // 1. 下载配置
      const configRes = await apiService.downloadConfig();
      if (configRes.success && configRes.data) {
        await invoke('sync_and_apply_config', { content: configRes.data });
      }

      // 2. 下载数据库 (Blob 转 Base64 传给 Rust 写入)
      const dbRes = await apiService.getSqliteDatabaseAsJson();
      if (dbRes.success && dbRes.data && dbRes.data.data) {
        console.log("正在同步数据库 JSON 数据...", dbRes.data);
        // 将整个数据对象转为 JSON 字符串传给 Rust
        const jsonString = JSON.stringify(dbRes.data);
        await invoke('sync_cloud_data', { jsonData: jsonString });
      }

      // 3. 下载剪贴板媒体文件 (镜像同步)
      const listRes = await apiService.getCloudFileList();
      if (listRes.success) {
        const serverPaths = listRes.data.map(item => item.relative_path);
        for (const item of listRes.data) {
          // 直接通过 Web URL 下载 Blob
          const fileUrl = ensureAbsoluteAvatarUrl(item.file);
          const fileBlob = await fetch(fileUrl, {
            headers: { 'Authorization': `Token ${localStorage.getItem('token')}` }
          }).then(r => r.blob());
          
          // 转 Base64 传给 Rust 保存
          const reader = new FileReader();
          reader.onload = async () => {
            const base64 = reader.result.split(',')[1];
            await invoke('save_clipboard_file', { relativePath: item.relative_path, base64Content: base64 });
          };
          reader.readAsDataURL(fileBlob);
        }
        // 清理本地不在云端的文件 (镜像清理)
        //await invoke('clean_local_files', { serverPaths });
      }

      lastSyncTime.value = Date.now();
      localStorage.setItem('lastSyncTime', lastSyncTime.value);
      if (!isSilent) showMessage('云端数据拉取成功', 'success');
    } catch (error) {
      console.error('拉取失败:', error);
      if (!isSilent) showMessage('同步拉取失败', 'error');
    } finally {
      isSyncing.value = false;
    }
  };

  // 辅助方法
  const getAIServiceName = (service) => {
    const serviceMap = {
      'openai': 'OpenAI',
      'claude': 'Claude', 
      'gemini': 'Gemini',
      'deepseek': 'DeepSeek',
      'custom': '自定义'
    }
    return serviceMap[service] || service
  }

  const getBackupFrequencyName = (frequency) => {
    const frequencyMap = {
      'daily': '每天',
      'weekly': '每周',
      'monthly': '每月'
    }
    return frequencyMap[frequency] || frequency
  }

  // 保存窗口状态到localStorage
  const saveWindowState = async () => {
    try {
      const scaleFactor = await currentWindow.scaleFactor()
      const position = await currentWindow.outerPosition()
      const size = await currentWindow.innerSize()
      
      const windowState = {
        x: position.x / scaleFactor,
        y: position.y / scaleFactor,
        width: size.width / scaleFactor,
        height: size.height / scaleFactor,
      }
      
      localStorage.setItem('preferencesWindowState', JSON.stringify(windowState))
      console.log('窗口状态已保存:', windowState)
    } catch (error) {
      console.error('保存窗口状态失败:', error)
    }
  }

  // 监听窗口关闭请求事件
  const setupWindowCloseListener = async () => {
    try {
      // 监听窗口关闭请求事件
      const unlistenCloseRequested = await currentWindow.onCloseRequested(async (event) => {
        if (firstCloseWindow) {
          // 阻止默认关闭行为，确保我们有时间保存状态
          event.preventDefault()
          firstCloseWindow = false
        }        
        
        console.log('窗口关闭请求，开始保存状态...')
        
        await saveWindowState()

        currentWindow.close()
      })
      
      return unlistenCloseRequested
      
    } catch (error) {
      console.error('设置窗口关闭监听器失败:', error)
      return null
    }
  }

  const base64ToBlob = (base64Content, mimeType) => {
      const byteString = atob(base64Content);
      const ab = new ArrayBuffer(byteString.length);
      const ia = new Uint8Array(ab);
      for (let i = 0; i < byteString.length; i++) {
          ia[i] = byteString.charCodeAt(i);
      }
      return new Blob([ab], { type: mimeType });
  }

  // 生命周期
  onMounted(async () => {
    // 检查本地存储中是否有用户信息
    try {
      const savedUser = localStorage.getItem('user')
      const savedToken = localStorage.getItem('token')
      if (savedUser) {
        const userData = JSON.parse(savedUser)
        userLoggedIn.value = true
        userEmail.value = userData.user.email || ''
        userInfo.username = userData.user.username || ''
        userInfo.email = userData.user.email || ''
        userInfo.bio = userData.user.bio || ''
        userInfo.avatar = ensureAbsoluteAvatarUrl(userData.user.avatar || '')
      }
    } catch (error) {
      console.error('加载用户信息失败:', error)
    }

    // 设置窗口关闭监听器
    unlistenCloseRequested = await setupWindowCloseListener()
  })

  return {
    // 状态
    activeNav,
    showToast,
    toastMessage,
    recordingShortcut,
    newIgnoredApp,
    userLoggedIn,
    userEmail,
    autostart,
    loading,
    errorMsg,
    successMsg,
    currentShortcut,
    shortcutManager,
    recordingShortcutType,
    lastSyncTime,
    lastSyncStatus,
    isSyncing,
    userInfo,
    navItems,
    settings,
    shortcutDisplayNames,
    shortcutKeys,

    // 注册登录相关状态
    showRegisterDialog,
    showLoginDialog,
    registerData,
    loginData,
    registerErrors,
    registerLoading,
    loginLoading,

    // 修改密码相关状态
    showChangePasswordDialog,
    changePasswordData,
    changePasswordErrors,
    changePasswordLoading,


    // 基础方法
    setActiveNav,
    goBack,
    login,
    logout,
    resetUserInfo,
    showMessage,

    // 注册登录方法
    handleRegister,
    handleLogin,
    openRegisterDialog,
    openLoginDialog,
    closeRegisterDialog,
    closeLoginDialog,
    updateUserInfo,

    // 修改密码方法
    handleChangePassword,
    openChangePasswordDialog,
    closeChangePasswordDialog,

    // 快捷键方法
    startRecording,
    cancelRecording,
    setShortcut,

    // 设置方法
    updateSetting,
    toggleOCRLanguage,
    changeStoragePath,

    // 数据管理方法
    clearAiHistory,
    exportData,
    importData,
    createBackup,

    // 隐私管理方法
    showPrivate,

    // 云端同步方法
    formatTime,
    manualSync,
    syncNow,
    checkSyncStatus,
    handleCloudPush,

    // 用户管理方法
    changeAvatar,
    deleteAccount,

    // 辅助方法
    getAIServiceName,
    getBackupFrequencyName
  }
}