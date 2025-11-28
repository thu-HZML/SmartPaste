// stores/settings.js
import { defineStore } from 'pinia'
import { ref, reactive } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSettingsStore = defineStore('settings', () => {
  // 默认设置（作为后备值）
  const defaultSettings = {
    autostart: true,
    tray_icon_visible: true,
    minimize_to_tray: false,
    auto_save: true,
    retention_days: 7,
    global_shortcut: "Alt+X",
    global_shortcut_2: "Alt+M",
    global_shortcut_3: "Alt+Shift+A",
    global_shortcut_4: "Ctrl+Shift+V",
    global_shortcut_5: "Ctrl+Shift+Delete",
    max_history_items: 500,
    ignore_short_text_len: 3,
    ignore_big_file_mb: 5,
    ignored_apps: [],
    auto_classify: true,
    ocr_auto_recognition: false,
    delete_confirmation: false,
    keep_favorites_on_delete: true,
    auto_sort: false,
    ai_enabled: false,
    ai_service: null,
    ai_api_key: null,
    ai_auto_tag: false,
    ai_auto_summary: false,
    ai_translation: false,
    ai_web_search: false,
    sensitive_filter: true,
    filter_passwords: true,
    filter_bank_cards: true,
    filter_id_cards: true,
    filter_phone_numbers: true,
    privacy_retention_days: 90,
    privacy_records: [],
    storage_path: "C:\\Users\\heyufei\\AppData\\Roaming\\com.tauri-app.desktop-pet",
    auto_backup: false,
    backup_frequency: "weekly",
    last_backup_path: null,
    cloud_sync_enabled: false,
    sync_frequency: "5min",
    sync_content_type: "onlytxt",
    encrypt_cloud_data: false,
    sync_only_wifi: true,
    username: null,
    email: null,
    bio: null,
    avatar_path: null,
    ocr_provider: null,
    ocr_languages: [],
    ocr_confidence_threshold: null,
    ocr_timeout_secs: null
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