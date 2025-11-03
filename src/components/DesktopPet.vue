<!-- src/components/DesktopPet.vue -->
<script setup>
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'

const appWindow = getCurrentWebviewWindow()
const position = ref({ x: 0, y: 0 })
const isDragging = ref(false)
const dragOffset = ref({ x: 0, y: 0 })

onMounted(() => {
  console.log('桌宠应用已启动')
  setupEventListeners()
})

onUnmounted(() => {
  cleanupEventListeners()
})

const handleMouseDown = (event) => {
  isDragging.value = true
  dragOffset.value = {
    x: event.clientX - position.value.x,
    y: event.clientY - position.value.y
  }
}

const handleMouseMove = (event) => {
  if (!isDragging.value) return
  position.value = {
    x: event.clientX - dragOffset.value.x,
    y: event.clientY - dragOffset.value.y
  }
}

const handleMouseUp = () => {
  isDragging.value = false
}

const startDragging = () => {
  appWindow.startDragging()
}

const handleContextMenu = async (event) => {
  event.preventDefault()
  // 可以在这里添加右键菜单逻辑
  console.log('右键菜单 - 可以跳转到设置页面')
  // 例如：router.push('/settings')
}

const setupEventListeners = () => {
  document.addEventListener('mousemove', handleMouseMove)
  document.addEventListener('mouseup', handleMouseUp)
}

const cleanupEventListeners = () => {
  document.removeEventListener('mousemove', handleMouseMove)
  document.removeEventListener('mouseup', handleMouseUp)
}
</script>

<template>
  <div 
    class="desktop-pet"
    :style="{
      position: 'fixed',
      left: `${position.x}px`,
      top: `${position.y}px`,
      width: '100%',
      height: '100%',
      zIndex: 9999,
      cursor: isDragging ? 'grabbing' : 'grab'
    }"
    @mousedown="startDragging"
    @contextmenu="handleContextMenu"
  >
    <div class="pet-container">
      <!-- 使用您放在 public/pet.png 的图片 -->
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
  user-select: none;
  pointer-events: auto;
}

.pet-container {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}

.pet-image {
  width: 100px;
  height: 100px;
  filter: drop-shadow(2px 2px 4px rgba(0,0,0,0.3));
  transition: transform 0.2s ease;
}

.pet-image:hover {
  transform: scale(1.1);
}

.desktop-pet:active {
  cursor: grabbing;
}
</style>