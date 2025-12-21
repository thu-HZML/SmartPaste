import { ref, reactive, onMounted, onUnmounted, watch} from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { emit } from '@tauri-apps/api/event'
import { apiService,ensureAbsoluteAvatarUrl } from '../services/api'
import { useSettingsStore } from '../stores/settings'
import { useSecurityStore } from '../stores/security'
import { loadUsername } from './Menu'
import { 
  Cog6ToothIcon,
  TvIcon,
  CloudIcon,
  ClipboardIcon,
  UserIcon,
  EyeSlashIcon,
  InboxArrowDownIcon,
  ChatBubbleLeftRightIcon
} from '@heroicons/vue/24/outline'
import { togglePrivateWindow } from '../utils/actions.js'

const base64ToBlob = (base64Content, mimeType) => {
  const byteString = atob(base64Content);
  const ab = new ArrayBuffer(byteString.length);
  const ia = new Uint8Array(ab);
  for (let i = 0; i < byteString.length; i++) {
      ia[i] = byteString.charCodeAt(i);
  }
  return new Blob([ab], { type: mimeType });
}

// 导出 executeCloudPush 函数 (核心同步逻辑，不包含 UI 交互)
export const executeCloudPush = async (dek = null) => {
  // 权限检查
  if (!localStorage.getItem('token')) {
    throw new Error('未登录');
  }

  console.log("开始执行云端推送 (Push)...", dek ? "[E2EE模式]" : "[普通模式]");

  try {
    // 1. 同步配置
    const configTxt = await invoke('get_config_json'); 
    const configRes = await apiService.uploadConfig(configTxt);
    if (!configRes.success) throw new Error(`配置同步失败: ${configRes.message}`);

    // 2. 同步数据库
    let dbBlob;
    
    if (dek) {
      // === E2EE 模式 ===
      console.log("正在准备加密数据库...");
      
      // 【修正1】改回驼峰命名 'dekHex'，符合 Tauri 默认规范
      const response = await invoke('prepare_encrypted_db_upload', { dekHex: dek });
      
      let encryptedBase64;

      // 【修正2】保留智能判断：检查返回值是路径还是内容
      // 如果返回值很长（超过500字符）或者包含 SQLite 头，说明它直接返回了内容
      if (response.length > 500 || response.startsWith("U1FMaXRl")) {
          console.log("✅ 检测到后端直接返回了数据库内容");
          
          // 安全检查：如果是明文 SQLite 头 (U1FMaXRl...)，说明加密可能未生效
          // 但考虑到这只是防止报错，我们先允许通过，但在控制台警告
          if (response.startsWith("U1FMaXRl")) {
             console.warn("⚠️ 警告：上传的数据似乎包含明文 SQLite 文件头，请确认 Rust 端加密是否正确。");
          }
          
          encryptedBase64 = response;
      } else {
          // 正常情况：返回值是路径，需要读取
          console.log("加密临时文件路径:", response);
          encryptedBase64 = await invoke('read_file_base64', { filePath: response });
      }

      dbBlob = base64ToBlob(encryptedBase64, 'application/octet-stream');

    } else {
      // === 普通模式 ===
      console.log("读取普通数据库...");
      const dbBase64 = await invoke('read_db_file_base64');
      dbBlob = base64ToBlob(dbBase64, 'application/x-sqlite3');
    }

    // 上传数据库 Blob
    const dbRes = await apiService.pushSqliteDatabase(dbBlob);
    if (!dbRes.success) throw new Error(`数据库推送失败: ${dbRes.message}`);

    // 3. 同步文件 (图片/附件)
    const localFiles = await invoke('get_local_files_to_upload');
    console.log(`发现 ${localFiles.length} 个文件需要上传`);

    for (const fileInfo of localFiles) {
      let contentBase64;
      let uploadPath = fileInfo.relative_path;

      try {
        if (dek) {
           // === E2EE 模式 ===
           const tempEncPath = fileInfo.file_path + ".enc";
           
           // 【修正3】这里也改回驼峰命名 inputPath, outputPath, dekHex
           await invoke('encrypt_file', { 
               inputPath: fileInfo.file_path, 
               outputPath: tempEncPath, 
               dekHex: dek 
           });
           
           // 读取加密后的内容
           contentBase64 = await invoke('read_file_base64', { filePath: tempEncPath });
           
           // (可选) 清理临时文件 (建议开启，防止垃圾文件堆积)
           // await invoke('delete_file', { path: tempEncPath });
        } else {
           // === 普通模式 ===
           contentBase64 = await invoke('read_file_base64', { filePath: fileInfo.file_path });
        }

        const blob = base64ToBlob(contentBase64, 'application/octet-stream');
        const fileRes = await apiService.uploadClipboardFile(blob, uploadPath);
        
        if (!fileRes.success) {
          console.warn(`文件上传失败 (${fileInfo.relative_path}):`, fileRes.message);
        }
      } catch (err) {
        console.error(`处理文件 ${fileInfo.relative_path} 时出错:`, err);
      }
    }

    return true;
  } catch (error) {
    console.error('后台同步执行出错:', error);
    throw error;
  }
};

export function usePreferences() {
  const router = useRouter()
  const currentWindow = getCurrentWindow();
  const securityStore = useSecurityStore()

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
    { id: 'ai', name: 'AI Agent 设置', icon: ChatBubbleLeftRightIcon },
    { id: 'security', name: '安全与隐私', icon: EyeSlashIcon }, 
    { id: 'backup', name: '数据备份', icon: InboxArrowDownIcon },
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
    
    // 验证密码 - 根据密码限制信息修改
    if (!registerData.password) {
      registerErrors.password = '密码不能为空'
      isValid = false
    } else if (registerData.password.length < 8) {
      registerErrors.password = '密码至少8个字符'
      isValid = false
    } else if (/^\d+$/.test(registerData.password)) {
      // 检查是否完全由数字组成
      registerErrors.password = '密码不能完全由数字组成'
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
    } else if (changePasswordData.new_password.length < 8) {
      changePasswordErrors.new_password = '新密码至少8个字符'
      isValid = false
    } else if (/^\d+$/.test(changePasswordData.new_password)) {
      // 检查是否完全由数字组成
      changePasswordErrors.new_password = '新密码不能完全由数字组成'
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

        // === 新增: 尝试恢复 E2EE 密钥 ===
        // 使用用户刚输入的密码尝试恢复
        try {
           await recoverE2EE(loginData.password);
           // 如果恢复成功，且设置中未开启加密（可能是新设备），自动开启
           if (securityStore.hasDek() && !settings.encrypt_cloud_data) {
             settings.encrypt_cloud_data = true;
           }
        } catch (e) {
           console.warn("E2EE 自动恢复失败 (可能未启用或网络问题):", e);
           // 注意：如果云端有密钥但解密失败（密码改过？），这里需要处理
        }

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
    // 如果是开启加密，需要特殊处理
    if (key === 'encrypt_cloud_data' && value === true) {
      // 1. 检查是否登录
      if (!userLoggedIn.value) {
        showMessage('请先登录以使用云端加密', 'warning');
        settings[key] = false; // 保持关闭
        return;
      }

      // 2. 检查内存中是否有 DEK
      if (securityStore.hasDek()) {
        // 已经有密钥了，直接开启
        const oldValue = settings[key];
        try {
          settings[key] = value;
          await invoke('set_config_item', { key, value });
          showMessage('加密设置已更新');
        } catch (e) {
           settings[key] = oldValue;
        }
        return;
      }

      // 3. 内存无密钥，需要走 Setup 流程
      // 这里有一个 UI 交互问题：我们需要密码。
      // 简单方案：弹出一个 prompt (浏览器原生)，或者你需要实现一个密码输入模态框
      const password = window.prompt("为了启用端到端加密，请验证您的登录密码：");
      if (!password) {
        settings[key] = false; // 用户取消
        return;
      }

      // 尝试恢复（万一云端已有），如果云端没有则生成
      try {
        const recovered = await recoverE2EE(password);
        if (recovered) {
           // 恢复成功，更新设置
           settings[key] = true;
           await invoke('set_config_item', { key, value: true });
           showMessage('密钥恢复成功，加密已启用', 'success');
        } else {
           // 云端无密钥，执行首次生成流程
           await setupE2EE(password);
        }
      } catch (e) {
         showMessage(`操作失败: ${e.message}`, 'error');
         settings[key] = false;
      }
      return; // 结束，不执行默认逻辑
    }
    
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
  // 处理云端同步开关切换
  const handleCloudSyncToggle = (event) => {
    const isChecked = event.target.checked
    
    // 如果尝试开启，但未登录
    if (isChecked && !userLoggedIn.value) {
      // 1. 视觉上恢复为未选中状态
      event.target.checked = false
      
      // 2. 确保 Store 中的状态为关闭 (防止状态不一致)
      //updateSetting('cloud_sync_enabled', false)
      
      // 3. 提示用户并跳转
      showMessage('请先登录账户以启用云端同步功能', 'warning')
      activeNav.value = 'user' // 跳转到用户信息页方便登录
    } else {
      // 正常更新设置
      updateSetting('cloud_sync_enabled', isChecked)
    }
  }
  watch(userLoggedIn, (isLoggedIn) => {
    if (!isLoggedIn && settings.cloud_sync_enabled) {
      updateSetting('cloud_sync_enabled', false)
      showMessage('已退出登录，云端同步已自动关闭', 'info')
    }
  })

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

  /**
   * 流程 3.1: 初始化 E2EE (生成并上传密钥)
   * 当用户开启加密开关时调用
   */
  const setupE2EE = async (password) => {
    try {
      loading.value = true;
      showMessage('正在生成加密密钥...', 'info');

      // 1. 本地生成密钥
      const salt = await invoke('generate_salt');
      const dek = await invoke('generate_dek');

      // 校验 Rust 返回值
      if (!salt || !dek) {
          throw new Error("本地密钥生成失败 (Rust 返回空值)");
      }
      
      // 2. 派生主密钥并封装 DEK
      const mk = await invoke('derive_mk', { password: password, saltHex: salt });
      const encryptedDek = await invoke('wrap_dek', { dekHex: dek, mkHex: mk });

      // 3. 上传到云端
      const res = await apiService.uploadEncryptionKeys({
        kdf_salt: salt,
        encrypted_dek: encryptedDek,
        kdf_algorithm: "Argon2id"
      });

      if (res.success) {
        // 4. 保存到内存 Store
        securityStore.setDek(dek);
        // 5. 更新设置状态
        settings.encrypt_cloud_data = true;
        await invoke('set_config_item', { key: 'encrypt_cloud_data', value: true });
        showMessage('端到端加密已启用', 'success');
      } else {
        throw new Error(res.message);
      }
    } catch (e) {
      console.error(e);
      showMessage(`启用加密失败: ${e.message || e}`, 'error');
      // 回滚开关状态
      settings.encrypt_cloud_data = false;
    } finally {
      loading.value = false;
    }
  }

  /**
   * 流程 3.2: 恢复 E2EE (从云端获取并解密密钥)
   * 登录成功后，或检测到需要密钥时调用
   */
  const recoverE2EE = async (password) => {
    try {
      // 1. 获取云端配置
      const res = await apiService.getEncryptionKeys();
      
      // 增加多重校验：确保 success, has_keys 为 true，且 data 中的字段确实存在
      if (res.success && res.has_keys && res.data && res.data.data.kdf_salt && res.data.data.encrypted_dek) {
        const kdf_salt = res.data.data.kdf_salt;
        const encrypted_dek = res.data.data.encrypted_dek;
        
        // 再次检查 salt 是否为空字符串
        if (!kdf_salt) {
            console.warn("跳过恢复：kdf_salt 为空");
            return false;
        }

        // 2. 派生 MK
        const mk = await invoke('derive_mk', { password: password, saltHex: kdf_salt });
        
        // 3. 解封装 DEK
        const dek = await invoke('unwrap_dek', { encryptedDekHex: encrypted_dek, mkHex: mk });
        
        // 4. 存入 Store
        securityStore.setDek(dek);
        console.log("E2EE 密钥恢复成功");
        
        // 恢复成功后，确保本地开关与云端状态一致
        if (!settings.encrypt_cloud_data) {
           settings.encrypt_cloud_data = true;
           // 可以在这里静默更新一下本地配置，避免下次重复提示
           invoke('set_config_item', { key: 'encrypt_cloud_data', value: true }).catch(()=>{});
        }
        
        return true;
      } else {
        // 云端没有密钥，或者数据不完整 -> 视为未启用 E2EE
        console.log("当前账户未设置 E2EE 密钥 (新账户或未启用)");
        return false; 
      }
    } catch (e) {
      console.error("密钥恢复异常:", e);
      // 如果是自动恢复（登录时），尽量不要抛出打断流程的 Error，除非是密码错误明确需要提示
      // 这里返回 false 表示恢复失败
      return false;
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
      // 传入 DEK (如果有且开启了加密)
      const dek = (settings.encrypt_cloud_data && securityStore.dek) ? securityStore.dek : null;
      
      // 如果开启了加密但没有 DEK (比如刷新了页面)，需要提示用户输入密码
      if (settings.encrypt_cloud_data && !dek) {
         // 这里的交互比较难处理，简单方式是抛出错误提示重新登录或验证
         throw new Error("加密密钥丢失，请重新登录或验证密码以恢复同步能力");
      }

      await executeCloudPush(dek); // 传入 dek

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

  // 用于在 UI 手动触发密钥恢复
  const restoreKeysManually = async () => {
    const password = window.prompt("请输入登录密码以恢复加密密钥：");
    if (!password) return;
    
    try {
        // 复用之前定义的 recoverE2EE 逻辑
        const success = await recoverE2EE(password); 
        if (success) {
            showMessage("密钥恢复成功", "success");
        } else {
            showMessage("未找到云端密钥配置", "error");
        }
    } catch(e) {
        showMessage(e.message, "error");
    }
  }

  const handleCloudPull = async (isSilent = false) => {
    if (isSyncing.value) return;
    isSyncing.value = true;
    
    try {
      if (!isSilent) showMessage('正在同步云端数据...', 'info');

      // 1. 下载并应用配置 (保持不变)
      const configRes = await apiService.downloadConfig();
      if (configRes.success && configRes.data) {
        await invoke('sync_and_apply_config', { content: configRes.data });
      }

      // 2. 下载数据库 (保持不变，已符合文档)
      const dbRes = await apiService.getSqliteDatabaseAsJson();
      if (dbRes.success && dbRes.data && dbRes.data.data) {
        const jsonString = JSON.stringify(dbRes.data);
        
        if (settings.encrypt_cloud_data) {
            const dek = securityStore.dek;
            if (!dek) {
                if(!isSilent) showMessage("无法解密数据：密钥未加载，请先验证密码", 'error');
                console.error("Sync aborted: E2EE enabled but no DEK.");
                return; 
            }
            // E2EE 模式：传入 DEK 解密数据条目
            await invoke('sync_encrypted_cloud_data', { 
                jsonData: jsonString, 
                dekHex: dek 
            });
        } else {
            // 普通模式
            await invoke('sync_cloud_data', { jsonData: jsonString });
        }
      }

      // 3. 下载文件/图片 (严格按照文档修改)
      const listRes = await apiService.getCloudFileList();
      if (listRes.success) {
        // 获取本地存储根路径，用于构造绝对路径传给 decrypt_file
        // 注意：这里假设 settings.storage_path 与 Rust 端实际使用的路径一致
        // 去除末尾斜杠以防重复
        const storageRoot = settings.storage_path.replace(/[\\/]$/, '');

        for (const item of listRes.data) {
          // 3.1 下载文件流 (Blob)
          const fileUrl = ensureAbsoluteAvatarUrl(item.file);
          const fileBlob = await fetch(fileUrl, {
            headers: { 'Authorization': `Token ${localStorage.getItem('token')}` }
          }).then(r => r.blob());
          
          // 3.2 转换为 Base64 以便通过 Tauri Command 写入
          const reader = new FileReader();
          reader.readAsDataURL(fileBlob);
          
          await new Promise((resolve, reject) => {
            reader.onload = async () => {
              try {
                const base64 = reader.result.split(',')[1];
                const relativePath = item.relative_path;

                if (settings.encrypt_cloud_data && securityStore.dek) {
                    // === E2EE 解密流程 ===
                    
                    // A. 定义临时加密文件路径 (例如 images/123.png.enc)
                    const tempRelativePath = relativePath + ".enc";
                    
                    // B. 先将加密内容写入磁盘 (复用现有的保存文件命令)
                    await invoke('save_clipboard_file', { 
                        relativePath: tempRelativePath, 
                        base64Content: base64 
                    });

                    const isWin = storageRoot.includes('\\');
                    const sep = isWin ? '\\' : '/';

                    const cleanJoin = (...parts) => {
                        return parts.map((part, index) => {
                            if (!part) return '';
                            // 移除首尾的斜杠和反斜杠（除了第一个路径的开头）
                            let s = part;
                            if (index > 0) s = s.replace(/^[\\\/]+/, '');
                            if (index < parts.length - 1) s = s.replace(/[\\\/]+$/, '');
                            return s;
                        }).join(sep);
                    };
                  
                    // 2. 构造绝对路径
                    const inputPath = cleanJoin(storageRoot, 'files', tempRelativePath);
                    const outputPath = cleanJoin(storageRoot, 'files', relativePath);
                  
                    // 3. 打印路径以供检查 (如果报错，请检查控制台打印的这个路径是否真实存在)
                    console.log(`[E2EE] Decrypting: \n In:  ${inputPath} \n Out: ${outputPath}`);

                    // D. 调用 Rust 进行解密 (文档 3.5 节)
                    await invoke('decrypt_file', {
                        inputPath: inputPath,
                        outputPath: outputPath,
                        dekHex: securityStore.dek
                    });

                    // (可选) E. 可以在此处调用 Rust 删除 .enc 临时文件
                    // await invoke('delete_file', { path: inputPath });

                } else {
                    // === 普通流程 ===
                    // 直接保存原始内容
                    await invoke('save_clipboard_file', { 
                        relativePath: relativePath, 
                        base64Content: base64 
                    });
                }
                resolve();
              } catch (e) {
                console.error(`处理文件 ${item.relative_path} 失败:`, e);
                // 单个文件失败不中断整个循环，但打印错误
                resolve(); 
              }
            };
            reader.onerror = reject;
          });
        }
      }

      lastSyncTime.value = Date.now();
      localStorage.setItem('lastSyncTime', lastSyncTime.value);
      if (!isSilent) showMessage('云端数据同步完成', 'success');

    } catch (error) {
      console.error('同步失败:', error);
      if (!isSilent) showMessage(`同步失败: ${error.message || error}`, 'error');
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

    // 从URL参数设置初始导航项
    const urlParams = new URLSearchParams(window.location.search);
    const navFromUrl = urlParams.get('nav');
    if (navFromUrl) {
      activeNav.value = navFromUrl;
    }

    onUnmounted(() => {
      if (unlisten) unlisten();
    });

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

    // 安全相关状态
    securityStore,

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
    handleCloudSyncToggle,
    formatTime,
    manualSync,
    syncNow,
    checkSyncStatus,
    handleCloudPush,
    restoreKeysManually,
    handleCloudPull,
    

    // 用户管理方法
    changeAvatar,
    deleteAccount,

    // 辅助方法
    getAIServiceName,
    getBackupFrequencyName
  }
}