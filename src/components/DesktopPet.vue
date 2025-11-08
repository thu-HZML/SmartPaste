// DesktopPet.vue - 简化版本
<script setup>
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { invoke } from '@tauri-apps/api/core'

const appWindow = getCurrentWebviewWindow()
const position = ref({ x: 0, y: 0 })
const isDragging = ref(false)
const dragOffset = ref({ x: 0, y: 0 })
const isHovering = ref(false)

const emit = defineEmits(['show-menu', 'hide-menu'])

// 用于延迟恢复穿透的定时器
let passthroughTimer = null

onMounted(async () => {
  console.log('[DesktopPet] mounted')
  setupEventListeners()

  // 初始化：关闭穿透，桌宠可点击
  try {
    await invoke('set_mouse_passthrough', { passthrough: false })
    console.log('[DesktopPet] 初始化：关闭穿透，桌宠可点击')
  } catch (e) {
    console.error('[DesktopPet] 初始化穿透失败', e)
  }

  // 初始位置设置为右下角
  const screenWidth = window.innerWidth
  const screenHeight = window.innerHeight
  position.value = {
    x: screenWidth - 170,
    y: screenHeight - 170
  }
})

onUnmounted(() => {
  cleanupEventListeners()
  if (passthroughTimer) clearTimeout(passthroughTimer)
})

// 关闭鼠标穿透（当鼠标在桌宠上时）
const disablePassthrough = async () => {
  if (passthroughTimer) {
    clearTimeout(passthroughTimer)
    passthroughTimer = null
  }
  
  if (!isHovering.value) {
    isHovering.value = true
    try {
      console.log('[DesktopPet] 尝试关闭穿透...')
      const result = await invoke('set_mouse_passthrough', { passthrough: false })
      console.log('[DesktopPet] 成功关闭穿透', result)
    } catch (e) {
      console.error('[DesktopPet] 关闭穿透失败:', e)
      console.error('错误详情:', e.message, e.stack)
    }
  }
}

// 开启鼠标穿透（当鼠标离开桌宠时）
const enablePassthrough = async () => {
  if (passthroughTimer) clearTimeout(passthroughTimer)
  
  passthroughTimer = setTimeout(async () => {
    if (isHovering.value && !isDragging.value) {
      isHovering.value = false
      try {
        console.log('[DesktopPet] 尝试开启穿透...')
        const result = await invoke('set_mouse_passthrough', { passthrough: true })
        console.log('[DesktopPet] 成功开启穿透', result)
      } catch (e) {
        console.error('[DesktopPet] 开启穿透失败:', e)
        // 详细显示错误信息
        console.error('错误详情:', e.message, e.stack)
      }
    }
    passthroughTimer = null
  }, 300)
}

// 拖拽逻辑
const handlePointerDown = (event) => {
  event.stopPropagation()
  disablePassthrough()
  isDragging.value = true
  dragOffset.value = {
    x: event.clientX - position.value.x,
    y: event.clientY - position.value.y
  }
  try {
    event.currentTarget.setPointerCapture(event.pointerId)
  } catch (e) {
    // ignore
  }
}

const handlePointerMove = (event) => {
  if (!isDragging.value) return
  event.stopPropagation()
  position.value = {
    x: event.clientX - dragOffset.value.x,
    y: event.clientY - dragOffset.value.y
  }
}

const handlePointerUp = (event) => {
  if (!isDragging.value) return
  event.stopPropagation()
  try {
    event.currentTarget.releasePointerCapture(event.pointerId)
  } catch (e) {
    // ignore
  }
  isDragging.value = false
  // 拖拽结束后不立即开启穿透，让用户有时间移开鼠标
  setTimeout(() => {
    enablePassthrough()
  }, 100)
}

// 鼠标进入桌宠区域
const handlePointerEnter = (event) => {
  event.stopPropagation()
  disablePassthrough()
}

// 鼠标离开桌宠区域
const handlePointerLeave = (event) => {
  event.stopPropagation()
  if (!isDragging.value) {
    enablePassthrough()
  }
}

// 点击打开菜单
const handleLeftClick = (event) => {
  event.stopPropagation()
  console.log('🖱️ 桌宠被点击')

  const rect = event.currentTarget.getBoundingClientRect()
  const menuPosition = {
    x: rect.right + 10,
    y: Math.max(10, rect.top)
  }

  emit('show-menu', menuPosition)
}

const handleContextMenu = (event) => {
  event.preventDefault()
  event.stopPropagation()
  console.log('右键菜单')
}

// 全局事件监听
const setupEventListeners = () => {
  document.addEventListener('pointermove', handlePointerMove)
  document.addEventListener('pointerup', handlePointerUp)
}

const cleanupEventListeners = () => {
  document.removeEventListener('pointermove', handlePointerMove)
  document.removeEventListener('pointerup', handlePointerUp)
}
</script>

<template>
  <div
    class="desktop-pet"
    :style="{
      left: `${position.x}px`,
      top: `${position.y}px`,
      cursor: isDragging ? 'grabbing' : 'grab'
    }"
    @pointerenter="handlePointerEnter"
    @pointerleave="handlePointerLeave"
    @pointerdown="handlePointerDown"
    @click="handleLeftClick"
    @contextmenu="handleContextMenu"
  >
    <div class="pet-container">
      <img
        src="/pet.png"
        alt="Desktop Pet"
        draggable="false"
        class="pet-image"
      />
    </div>
  </div>
</template>

<style scoped>
.desktop-pet {
  position: fixed;
  width: 150px;
  height: 150px;
  z-index: 9999;
  user-select: none;
  pointer-events: auto;
  background: transparent;
}

.pet-container {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
}

.pet-image {
  width: 100px;
  height: 100px;
  filter: drop-shadow(2px 2px 4px rgba(0, 0, 0, 0.3));
  transition: transform 0.2s ease;
  background: transparent;
}

.pet-image:hover {
  transform: scale(1.1);
}

.desktop-pet:active {
  cursor: grabbing;
}
</style>