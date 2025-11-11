<!-- App.vue -->
<template>
  <div class="app-container">
    <!-- 路由视图 - 用于显示设置页面 -->
    <router-view v-if="$route.path !== '/'" />
    
    <!-- 桌宠组件（在首页时显示） -->
    <DesktopPet 
      v-if="$route.path === '/'"
      @show-menu="handleShowMenu"
      @hide-menu="handleHideMenu"
    />
    
    <!-- 剪贴板菜单覆盖层 -->
    <div 
      v-if="showClipboardMenu && $route.path === '/'"
      class="clipboard-menu-overlay"
      @click="handleHideMenu"
    >
      <div 
        class="clipboard-menu-container"
        :style="{
          left: `${menuPosition.x}px`,
          top: `${menuPosition.y}px`
        }"
        @click.stop
      >
        <ClipboardApp />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import DesktopPet from './components/DesktopPet.vue'
import ClipboardApp from './components/ClipboardApp.vue'

const router = useRouter()
const route = useRoute()
const showClipboardMenu = ref(false)
const menuPosition = ref({ x: 0, y: 0 })

// 监听路由变化，如果跳转到设置页面，隐藏菜单
watch(() => route.path, (newPath) => {
  if (newPath !== '/' && showClipboardMenu.value) {
    showClipboardMenu.value = false
  }
})

// 处理显示菜单
const handleShowMenu = (position) => {
  menuPosition.value = position
  showClipboardMenu.value = true
  
  // 调整菜单位置，确保不会超出屏幕
  adjustMenuPosition()
}

// 处理隐藏菜单
const handleHideMenu = () => {
  showClipboardMenu.value = false
}

// 调整菜单位置
const adjustMenuPosition = () => {
  setTimeout(() => {
    const menuElement = document.querySelector('.clipboard-menu-container')
    if (!menuElement) return
    
    const rect = menuElement.getBoundingClientRect()
    const screenWidth = window.innerWidth
    const screenHeight = window.innerHeight
    
    let adjustedX = menuPosition.value.x
    let adjustedY = menuPosition.value.y
    
    // 如果菜单右侧超出屏幕，向左调整
    if (rect.right > screenWidth) {
      adjustedX = screenWidth - rect.width - 20
    }
    
    // 如果菜单底部超出屏幕，向上调整
    if (rect.bottom > screenHeight) {
      adjustedY = screenHeight - rect.height - 20
    }
    
    // 如果调整了位置，更新菜单位置
    if (adjustedX !== menuPosition.value.x || adjustedY !== menuPosition.value.y) {
      menuPosition.value = { x: adjustedX, y: adjustedY }
    }
  }, 0)
}

// 监听窗口大小变化，重新调整菜单位置
onMounted(() => {
  window.addEventListener('resize', adjustMenuPosition)
  console.log('🖥️ 全屏透明窗口已启动')
})

onUnmounted(() => {
  window.removeEventListener('resize', adjustMenuPosition)
})
</script>

<style>
/* 全局样式 - 确保全屏透明 */

html, body {
  margin: 0;
  padding: 0;
  background: transparent;
  overflow: hidden;
  width: 100vw;
  height: 100vh;
  
}

#app {
  width: 100%;
  height: 100%;
  background: transparent;
}
*/
/* 应用容器 */

.app-container {
  width: 100%;
  height: 100%;
  position: relative;
  background: transparent;
}

/* 菜单覆盖层 */

.clipboard-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 9998;
  background: transparent;
  pointer-events: auto;
}

/* 菜单容器 */

.clipboard-menu-container {
  position: fixed;
  z-index: 10000;
  animation: slideIn 0.2s ease-out;
  width: 450px;
  max-height: 600px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
  border: 1px solid #e1e8ed;
  overflow: hidden;
  pointer-events: auto;
}

/* 菜单动画 */

@keyframes slideIn {
  from {
    opacity: 0;
    transform: translateX(-10px) scale(0.95);
  }
  to {
    opacity: 1;
    transform: translateX(0) scale(1);
  }
}


/* 确保菜单中的滚动条样式 */

.clipboard-menu-container ::-webkit-scrollbar {
  width: 6px;
}

.clipboard-menu-container ::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 3px;
}

.clipboard-menu-container ::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 3px;
}

.clipboard-menu-container ::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}

</style>