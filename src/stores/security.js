// src/stores/security.js
import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSecurityStore = defineStore('security', () => {
  const dek = ref(null)

  // 核心：从后端拉取状态
  const initFromBackend = async () => {
    try {
      const savedDek = await invoke('get_dek_state')
      if (savedDek) {
        dek.value = savedDek
        console.log("已从 Rust 后端恢复密钥")
      }
    } catch (e) {
      console.error("初始化密钥失败:", e)
    }
  }

  const setDek = async (key) => {
    dek.value = key
    // 同步到 Rust 后端内存
    await invoke('set_dek_state', { key })
  }

  const clearDek = async () => {
    dek.value = null
    // 通知后端清空
    await invoke('clear_dek_state')
  }

  const hasDek = () => {
    return !!dek.value
  }

  return {
    dek,
    setDek,
    clearDek,
    hasDek,
    initFromBackend
  }
})