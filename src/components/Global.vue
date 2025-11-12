<template>
  <div class="app-container">  
    <!-- 桌宠组件（在首页时显示） -->
    <DesktopPet 
      @show-menu="handleShowMenu"
      @hide-menu="handleHideMenu"
    />
    
    <!-- 剪贴板菜单覆盖层 -->
    <div 
      v-if="showClipboardMenu"
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
import { ref, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import DesktopPet from './DesktopPet.vue'
import ClipboardApp from './ClipboardApp.vue'

const showClipboardMenu = ref(false)
const menuPosition = ref({ x: 0, y: 0 })
const desktopPetRef = ref(null)
const interactiveElements = ref([])

const router = useRouter()

// 检测鼠标位置并动态调整穿透
const setupMouseTracking = () => {
  const checkMousePosition = async (event) => {
    if (showClipboardMenu.value) {
      // 菜单显示时，只穿透菜单外的区域
      const menuElement = document.querySelector('.clipboard-menu-container')
      if (menuElement) {
        const menuRect = menuElement.getBoundingClientRect()
        const isInMenu = 
          event.clientX >= menuRect.left &&
          event.clientX <= menuRect.right &&
          event.clientY >= menuRect.top &&
          event.clientY <= menuRect.bottom
        
        const petElement = desktopPetRef.value?.$el
        let isInPet = false
        
        if (petElement) {
          const petRect = petElement.getBoundingClientRect()
          isInPet = 
            event.clientX >= petRect.left &&
            event.clientX <= petRect.right &&
            event.clientY >= petRect.top &&
            event.clientY <= petRect.bottom
        }
        
        // 如果在菜单或桌宠内，不穿透；否则穿透
        await setWindowMousePenetration(!(isInMenu || isInPet))
      }
    } else {
      // 菜单不显示时，只穿透桌宠外的区域
      const petElement = desktopPetRef.value?.$el
      if (petElement) {
        const petRect = petElement.getBoundingClientRect()
        const isInPet = 
          event.clientX >= petRect.left &&
          event.clientX <= petRect.right &&
          event.clientY >= petRect.top &&
          event.clientY <= petRect.bottom
        
        // 如果在桌宠内，不穿透；否则穿透
        await setWindowMousePenetration(!isInPet)
      } else {
        // 没有找到桌宠元素，默认穿透
        console.error('未找到桌宠元素:', desktopPetRef.value)
        await setWindowMousePenetration(true)
      }
    }
  }

  document.addEventListener('mousemove', checkMousePosition)
  return () => {
    document.removeEventListener('mousemove', checkMousePosition)
  }
}

// 设置窗口鼠标穿透
const setWindowMousePenetration = async (enabled) => {
  try {
    if (window.__TAURI__) {
      const { appWindow } = await import('@tauri-apps/api/window')
      await appWindow.setIgnoreCursorEvents(enabled)
    }
  } catch (error) {
    console.error('设置鼠标穿透失败:', error)
  }
}

// 处理显示菜单
const handleShowMenu = async (position) => {
  menuPosition.value = position
  showClipboardMenu.value = true
  
  // 调整菜单位置，确保不会超出屏幕
  adjustMenuPosition()
}

// 处理隐藏菜单
const handleHideMenu = async () => {
  showClipboardMenu.value = false
  // router.push('/clipboardapp')
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

let removeMouseTracker = null

// 监听窗口大小变化，重新调整菜单位置
onMounted(async () => {
  window.addEventListener('resize', adjustMenuPosition)

  await nextTick()
  removeMouseTracker = setupMouseTracking()

  console.log('🖥️ 全屏透明窗口已启动')
})

onUnmounted(() => {
  window.removeEventListener('resize', adjustMenuPosition)
  if (removeMouseTracker) {
    removeMouseTracker()
  }
  // 清理时禁用穿透
  setWindowMousePenetration(false)
})
</script>

<style scoped>
/* 应用容器 - 全屏、无边框、透明的容器 */
.app-container {
  width: 100%;
  height: 100%;
  position: relative;
  background: transparent;
  pointer-events: none; /* 默认启用鼠标穿透 */
}

/* 桌宠组件 - 始终可交互 */
.app-container > :first-child {
  pointer-events: auto;
  z-index: 10;
  position: relative;
}

/* 菜单覆盖层 */
.clipboard-menu-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100vw;
  height: 100vh;
  z-index: 5;
  background: transparent;
  pointer-events: none; /* 允许点击穿透到下层 */
}

/* 菜单容器 */
.clipboard-menu-container {
  position: fixed;
  z-index: 6;
  animation: slideIn 0.2s ease-out;
  width: 450px;
  max-height: 600px;
  border-radius: 12px;
  box-shadow: 0 10px 30px rgba(0, 0, 0, 0.2);
  border: 1px solid #e1e8ed;
  overflow-y: scroll;
  pointer-events: auto;
  background: white;
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
  width: 1px;
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