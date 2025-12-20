// src/stores/security.js
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useSecurityStore = defineStore('security', () => {
  // DEK 仅保存在内存中，刷新页面即丢失，保证安全
  const dek = ref(null)

  const setDek = (key) => {
    dek.value = key
  }

  const clearDek = () => {
    dek.value = null
  }

  const hasDek = () => {
    return !!dek.value
  }

  return {
    dek,
    setDek,
    clearDek,
    hasDek
  }
})