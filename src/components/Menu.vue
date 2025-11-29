<template>
  <div class="main-menu">
    <!-- 用户信息区域 -->
    <div class="flex-row items-center user-section">
      <div class="user-avatar">
        <UserIcon class="icon-default" />
      </div>
      <div class="flex-col user-info">
        <span class="user-name">用户123</span>
      </div>
    </div>

    <!-- 第一行：历史记录、收藏夹、项目时间轴 -->
    <div class="flex-row menu-row">
      <div class="menu-item" @click="openHistory">
        <div class="menu-icon">
          <ClipboardDocumentListIcon class="icon-default" />
        </div>
        <span class="menu-text">历史记录</span>
      </div>
      <div class="menu-item" @click="openFavorites">
        <div class="menu-icon">
          <StarIcon class="icon-default" />
        </div>
        <span class="menu-text">收藏夹</span>
      </div>
      <div class="menu-item" @click="openAIAssistant">
        <div class="menu-icon">
          <img src="../assets/deepseek.svg" alt="AI助手" class="svg-icon" />
        </div>
        <span class="menu-text">AI助手</span>
      </div>
    </div>

    <!-- 第二行：设置、云端、AI助手 -->
    <div class="flex-row menu-row">
      <div class="menu-item" @click="openSettings">
        <div class="menu-icon">
          <Cog6ToothIcon class="icon-default" />
        </div>
        <span class="menu-text">设置</span>
      </div>
      <div class="menu-item" @click="openCloud">
        <div class="menu-icon">
          <CloudIcon class="icon-default" />
        </div>
        <span class="menu-text">云端</span>
      </div>
      <div class="menu-item" @click="openHelp">
        <div class="menu-icon">
          <QuestionMarkCircleIcon class="icon-default" />
        </div>
        <span class="menu-text">帮助</span>
      </div>
    </div>
  </div>
</template>

<script setup>
import { getCurrentWindow } from '@tauri-apps/api/window'
import { 
  BeakerIcon,
  Cog6ToothIcon,
  ArrowPathIcon,
  CloudIcon,
  StarIcon,
  ClipboardDocumentListIcon,
  QuestionMarkCircleIcon,
  Square2StackIcon,
  ChartBarIcon,
  UserIcon
 } from '@heroicons/vue/24/outline'
import { useMenuFunctions } from '../composables/Menu'

const currentWindow = getCurrentWindow()

// 使用函数
const { 
  openHistory, 
  openFavorites, 
  openSettings, 
  openCloud, 
  openAIAssistant, 
  openHelp 
} = useMenuFunctions()
</script>

<style scoped>
.main-menu {
  width: 100%;
  height: 100vh;
  background: #f8f9fa;
  padding: 1.5rem;
  display: flex;
  flex-direction: column;
  align-items: center;
}

/* 用户信息区域 */
.user-section {
  padding: 1rem 1.5rem;
  background: #f8f9fa;
  border-radius: 0.75rem;
  border: none;
}

.user-avatar {
  width: 3.5rem;
  height: 3.5rem;
  background: #e9ecef;
  border-radius: 50%;
  margin-right: 1rem;
  display: flex;
  align-items: center;
  justify-content: center;
}

.user-name {
  font-size: 1.1rem;
  font-weight: 600;
  color: #212529;
  font-family: 'Microsoft YaHei', sans-serif;
}

/* 菜单行 */
.menu-row {
  width: 100%;
  justify-content: space-between;
  margin-bottom: 1.5rem;
}

/* 菜单项 */
.menu-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 1rem 0.5rem;
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.3s ease;
  flex: 1;
  margin: 0 0.25rem;
  background: #f8f9fa;
  border: none;
  box-shadow: none;
  min-height: 3rem;
}

.menu-item:hover {
  background: #e9ecef;
  transform: none;
  box-shadow: none;
}

/* 菜单图标 */
.menu-icon {
  width: 3rem;
  height: 3rem;
  border-radius: 0.6rem;
  margin-bottom: 0.75rem;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #e9ecef;
}

/* 菜单文字 */
.menu-text {
  font-size: 0.9rem;
  color: #495057;
  font-weight: 500;
  font-family: 'Microsoft YaHei', sans-serif;
  text-align: center;
}

/* 图标样式 */
.icon-default {
  width: 1.5rem;
  height: 1.5rem;
  color: #6c757d;
}

.menu-item:hover .icon-default {
  color: #495057;
}

/* 空菜单项样式 */
.menu-item:empty {
  visibility: hidden;
  background: transparent;
}

.svg-icon {
  width: 1.5rem;
  height: 1.5rem;
  color: #6c757d; /* 如果需要改变颜色 */
}

.menu-item:hover .svg-icon {
  color: #495057;
}

</style>