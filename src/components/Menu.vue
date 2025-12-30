<template>
  <div class="main-menu">
    <!-- 用户信息区域 -->
    <div class="flex-row items-center user-section"@click="openSettings('user')" 
      style="cursor: pointer;">
      <div class="user-avatar">
        <!-- 如果有用户头像则显示，否则显示默认图标 -->
        <img 
          v-if="userAvatar" 
          :src="userAvatar" 
          alt="用户头像" 
          class="avatar-img"
        >
        <UserIcon v-else class="icon-default" />
      </div>
      <div class="flex-col user-info">
        <span class="user-name">{{ username }}</span>
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
      <div class="menu-item" @click="openSettings('cloud')">
        <div class="menu-icon">
          <CloudIcon class="icon-default" />
        </div>
        <span class="menu-text">云端</span>
      </div>
    </div>   

    <!-- 第二行：设置、云端、AI助手 -->
    <div class="flex-row menu-row">
      <div class="menu-item" @click="openSettings('general')">
        <div class="menu-icon">
          <Cog6ToothIcon class="icon-default" />
        </div>
        <span class="menu-text">设置</span>
      </div>
      <div class="menu-item" @click="openSettings('help')"">
        <div class="menu-icon">
          <QuestionMarkCircleIcon class="icon-default" />
        </div>
        <span class="menu-text">帮助</span>
      </div>
      <!-- 空占位，保持与第一行对齐 -->
      <div class="menu-item"></div>
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
import { useMenuFunctions, useUsername} from '../composables/Menu'

const currentWindow = getCurrentWindow()
const { username,userAvatar } = useUsername();

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
  align-items: flex-start;
}

/* 用户信息区域 */
.user-section {
  padding: 0.3rem 0.3rem;
  background: #f8f9fa;
  border-radius: 0.75rem;
  border: none;
  margin-bottom: 1rem;
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
  overflow: hidden; /* 关键：隐藏超出部分 */
  border: 2px solid #dee2e6; /* 添加边框增强圆形效果 */
  flex-shrink: 0; /* 防止头像容器被压缩 */
  position: relative; /* 为内部元素定位做准备 */
}

.user-name {
  font-size: 1.1rem;
  font-weight: 600;
  color: #212529;
  font-family: 'Microsoft YaHei', sans-serif;
}

/* 头像图片样式  */
.avatar-img {
  width: 100%; /* 填满父容器 */
  height: 100%; 
  object-fit: cover; 
  object-position: center center; /* 图片居中显示 */
  display: block; /* 避免图片下方有间隙 */
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
  padding: 0.2rem 0.25rem;
  border-radius: 0.75rem;
  cursor: pointer;
  transition: all 0.3s ease;
  flex: 1;
  margin: 0 0.2rem;
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

/* 可以添加悬停效果让头像更有交互性 */
.user-avatar:hover {
  transform: scale(1.05);
  transition: transform 0.3s ease;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

/* 如果需要为头像添加光泽效果 */
.user-avatar::after {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  border-radius: 50%;
  border: 2px solid rgba(255, 255, 255, 0.3);
  pointer-events: none;
}

</style>