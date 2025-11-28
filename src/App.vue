<!-- App.vue -->
<template>
  <router-view></router-view>
</template>

<script setup>
// 这里可以放一些全局的逻辑
import { onMounted } from 'vue'
import { useSettingsStore } from './stores/settings'

const settingsStore = useSettingsStore()

onMounted(async () => {
  // 应用启动时初始化配置
  await settingsStore.initializeSettings()
  console.log('应用配置已初始化')
})
</script>

<style>
/* 全局样式可以放在这里 */
</style>