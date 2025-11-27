// stores/settings.js
import { defineStore } from 'pinia'
import { ref, reactive } from 'vue'

export const useSettingsStore = defineStore('settings', () => {
  // 基本设置
  const autoStart = ref(true)
  const showTrayIcon = ref(true)
  const showMinimizeTrayIcon = ref(true)
  const autoSave = ref(true)
  const retentionDays = ref(30)
  const maxHistoryItems = ref(100)
  const ignoreShortText = ref(3)
  const ignoreBigFile = ref(5)
  const ignoredApps = ref(['密码管理器', '银行应用'])
  const previewLength = ref(115)
  const cloudSync = ref(true)
  const syncFrequency = ref('realtime')
  const syncContantType = ref('onlytxt')
  const syncOnlyWifi = ref(true)
  const encryptCloudData = ref(true)

  // 剪贴板参数设置
  const autoClassify = ref(true)
  const ocrAutoRecognition = ref(true)
  const deleteConfirmation = ref(true)
  const keepFavorites = ref(true)
  const autoSort = ref(true)

  // OCR设置
  const ocrProvider = ref('auto')
  const ocrLanguages = ref(['chi_sim', 'eng'])
  const ocrConfidenceThreshold = ref(80)
  const ocrTimeoutSecs = ref(30)
  
  // AI Agent 设置
  const aiEnabled = ref(false)
  const aiService = ref('openai')
  const aiApiKey = ref('')
  const aiAutoTag = ref(true)
  const aiAutoSummary = ref(true)
  const aiTranslation = ref(true)
  const aiWebSearch = ref(false)
  
  // 安全与隐私
  const sensitiveFilter = ref(true)
  const filterPasswords = ref(true)
  const filterBankCards = ref(true)
  const filterIDCards = ref(true)
  const filterPhoneNumbers = ref(true)
  const privacyRetentionDays = ref('7')
  
  // 数据备份
  const dataStoragePath = ref('')
  const autoBackup = ref(true)
  const backupFrequency = ref('weekly')

  // 快捷键设置
  const shortcuts = reactive({
    toggleWindow: '',
    pasteWindow: '',
    AIWindow: '',
    quickPaste: '',
    clearHistory: ''
  })

  // 批量更新设置的方法
  const updateSettings = (newSettings) => {
    Object.keys(newSettings).forEach(key => {
      if (key in shortcuts && typeof newSettings[key] === 'object') {
        Object.assign(shortcuts[key], newSettings[key])
      } else if (key in $state) {
        $state[key] = newSettings[key]
      }
    })
  }

  // 重置为默认设置
  const resetToDefaults = () => {
    autoStart.value = true
    showTrayIcon.value = true
    showMinimizeTrayIcon.value = true
    autoSave.value = true
    retentionDays.value = 30
    maxHistoryItems.value = 100
    ignoreShortText.value = 3
    ignoreBigFile.value = 5
    ignoredApps.value = ['密码管理器', '银行应用']
    previewLength.value = 115
    cloudSync.value = true
    syncFrequency.value = 'realtime'
    syncContantType.value = 'onlytxt'
    syncOnlyWifi.value = true
    encryptCloudData.value = true
    autoClassify.value = true
    ocrAutoRecognition.value = true
    deleteConfirmation.value = true
    keepFavorites.value = true
    autoSort.value = true
    ocrProvider.value = 'auto'
    ocrLanguages.value = ['chi_sim', 'eng']
    ocrConfidenceThreshold.value = 80
    ocrTimeoutSecs.value = 30
    aiEnabled.value = false
    aiService.value = 'openai'
    aiApiKey.value = ''
    aiAutoTag.value = true
    aiAutoSummary.value = true
    aiTranslation.value = true
    aiWebSearch.value = false
    sensitiveFilter.value = true
    filterPasswords.value = true
    filterBankCards.value = true
    filterIDCards.value = true
    filterPhoneNumbers.value = true
    privacyRetentionDays.value = '7'
    dataStoragePath.value = ''
    autoBackup.value = true
    backupFrequency.value = 'weekly'
    
    Object.keys(shortcuts).forEach(key => {
      shortcuts[key] = ''
    })
  }

  return {
    // 基本设置
    autoStart,
    showTrayIcon,
    showMinimizeTrayIcon,
    autoSave,
    retentionDays,
    maxHistoryItems,
    ignoreShortText,
    ignoreBigFile,
    ignoredApps,
    previewLength,
    cloudSync,
    syncFrequency,
    syncContantType,
    syncOnlyWifi,
    encryptCloudData,

    // 剪贴板参数设置
    autoClassify,
    ocrAutoRecognition,
    deleteConfirmation,
    keepFavorites,
    autoSort,

    // OCR设置
    ocrProvider,
    ocrLanguages,
    ocrConfidenceThreshold,
    ocrTimeoutSecs,
    
    // AI Agent 设置
    aiEnabled,
    aiService,
    aiApiKey,
    aiAutoTag,
    aiAutoSummary,
    aiTranslation,
    aiWebSearch,
    
    // 安全与隐私
    sensitiveFilter,
    filterPasswords,
    filterBankCards,
    filterIDCards,
    filterPhoneNumbers,
    privacyRetentionDays,
    
    // 数据备份
    dataStoragePath,
    autoBackup,
    backupFrequency,

    // 快捷键
    shortcuts,

    // 方法
    updateSettings,
    resetToDefaults
  }
})