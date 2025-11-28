// stores/settings.js
import { defineStore } from 'pinia'
import { ref, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSettingsStore = defineStore('settings', () => {
  // 默认设置（作为后备值）
  const defaultSettings = {
    autoStart: true,
    showTrayIcon: true,
    showMinimizeTrayIcon: true,
    autoSave: true,
    retentionDays: 30,
    maxHistoryItems: 100,
    ignoreShortText: 3,
    ignoreBigFile: 5,
    ignoredApps: ['密码管理器', '银行应用'],
    previewLength: 115,
    cloudSync: true,
    syncFrequency: 'realtime',
    syncContantType: 'onlytxt',
    syncOnlyWifi: true,
    encryptCloudData: true,
    autoClassify: true,
    ocrAutoRecognition: true,
    deleteConfirmation: true,
    keepFavorites: true,
    autoSort: true,
    ocrProvider: 'auto',
    ocrLanguages: ['chi_sim', 'eng'],
    ocrConfidenceThreshold: 80,
    ocrTimeoutSecs: 30,
    aiEnabled: false,
    aiService: 'openai',
    aiApiKey: '',
    aiAutoTag: true,
    aiAutoSummary: true,
    aiTranslation: true,
    aiWebSearch: false,
    sensitiveFilter: true,
    filterPasswords: true,
    filterBankCards: true,
    filterIDCards: true,
    filterPhoneNumbers: true,
    privacyRetentionDays: '7',
    dataStoragePath: '',
    autoBackup: true,
    backupFrequency: 'weekly',
    global_shortcut: '',
    global_shortcut_2: '',
    global_shortcut_3: '',
    global_shortcut_4: '',
    global_shortcut_5: ''
  }

  // 创建响应式设置对象，初始值为默认设置
  const settings = reactive({ ...defaultSettings })

  // 从后端加载配置的初始化函数
  const initializeSettings = async () => {
    try {
      console.log('正在从后端加载配置...')
      
      // 调用后端获取配置
      const configJson = await invoke('get_config_json')
      
      if (configJson && configJson.trim() !== '') {
        // 解析后端返回的配置
        const backendConfig = JSON.parse(configJson)
        
        // 将后端配置合并到当前设置中
        Object.assign(settings, backendConfig)
        
        console.log('配置加载成功:', backendConfig)
        return true
      } else {
        console.warn('后端返回空配置，使用默认设置')
        return false
      }
    } catch (error) {
      console.error('从后端加载配置失败:', error)
      // 加载失败时使用默认设置
      Object.assign(settings, defaultSettings)
      return false
    }
  }

  // 重置为默认设置
  const resetToDefaults = () => {
    Object.assign(settings, defaultSettings)
  }

  return {
    settings,
    initializeSettings,
    resetToDefaults
  }
})