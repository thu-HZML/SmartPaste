<script setup>
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
import { windowInstances, 
  toggleClipboardWindow, 
  updateMainWindowPosition, 
  toggleMenuWindow,
  updateMenuWindowPosition,
  updateMenuWindowPositionRealTime,  // 新增实时更新函数
  hasMenuWindow as checkMenuWindowExists
} from '../utils/actions.js'
import { listen, emit } from '@tauri-apps/api/event'

const isHovering = ref(false)
const hasClipboardWindow = ref(false)
const hasMenuWindow = ref(false)
const isDragging = ref(false)
const dragStartPos = ref({ x: 0, y: 0 })
const windowStartPos = ref({ x: 0, y: 0 })
const currentWindow = getCurrentWindow();
const scaleFactor = ref(1.486)
const allowClickPet = ref(true)

// 存储当前窗口位置
const currentPosition = ref({ x: 0, y: 0 })


let clickPetTimeout = null
let positionUpdateInterval = null
let dragUpdateInterval = null  // 新增：拖拽时的更新间隔

onMounted(async () => {
  console.log('[DesktopPet] mounted')
  try {
    await currentWindow.setSize(new LogicalSize(150, 95));
    await currentWindow.setPosition(new LogicalPosition(1550, 800))
    const actualScaleFactor = await currentWindow.scaleFactor();
    console.log('系统缩放比例:', actualScaleFactor);
    scaleFactor.value = actualScaleFactor;
    
    // 初始化位置
    const position = await currentWindow.outerPosition()
    currentPosition.value = {
      x: Math.round(position.x / scaleFactor.value),
      y: Math.round(position.y / scaleFactor.value)
    }
    updateMainWindowPosition(currentPosition.value, { width: 120, height: 120 })
    
    // 启动位置更新监听
    startPositionTracking()
  } catch (error) {
    console.error('设置窗口大小失败:', error)
  }
})

onUnmounted(() => {
  stopPositionTracking()
  stopDragTracking()  // 新增：停止拖拽跟踪
  cleanupEventListeners()
})

// 启动位置跟踪（常规更新）
const startPositionTracking = () => {
  positionUpdateInterval = setInterval(async () => {
    // 只在有菜单窗口且不在拖拽状态下更新
    if (hasMenuWindow.value && !isDragging.value) {
      await updateWindowPosition()
      await updateMenuWindowPosition()
    }
  }, 500)  // 常规更新频率可以低一些
}

// 停止位置跟踪
const stopPositionTracking = () => {
  if (positionUpdateInterval) {
    clearInterval(positionUpdateInterval)
    positionUpdateInterval = null
  }
}

// 启动拖拽跟踪（高频更新）
const startDragTracking = () => {
  dragUpdateInterval = setInterval(async () => {
    if (hasMenuWindow.value && isDragging.value) {
      await updateWindowPosition()
      await updateMenuWindowPositionRealTime()  // 使用实时更新函数
    }
  }, 50)  // 拖拽时高频更新，50ms一次
}

// 停止拖拽跟踪
const stopDragTracking = () => {
  if (dragUpdateInterval) {
    clearInterval(dragUpdateInterval)
    dragUpdateInterval = null
  }
}

// 实时更新窗口位置
const updateWindowPosition = async () => {
  try {
    const position = await currentWindow.outerPosition()
    const newPosition = {
      x: Math.round(position.x / scaleFactor.value),
      y: Math.round(position.y / scaleFactor.value)
    }
    
    // 只有位置发生变化时才更新
    if (newPosition.x !== currentPosition.value.x || newPosition.y !== currentPosition.value.y) {
      currentPosition.value = newPosition
      updateMainWindowPosition(currentPosition.value, { width: 120, height: 120 })
      console.log('📍 主窗口位置更新:', currentPosition.value)
    }
  } catch (error) {
    console.error('更新窗口位置失败:', error)
  }
}

const handlePointerDown = async (event) => {
  event.stopPropagation()

  try {
    const physicalPosition = await currentWindow.outerPosition()
    windowStartPos.value = {
      x: Math.round(physicalPosition.x / scaleFactor.value),
      y: Math.round(physicalPosition.y / scaleFactor.value)
    }
  } catch (error) {
    console.error('获取窗口位置失败:', error)
  }
  
  dragStartPos.value = {
    x: event.screenX,
    y: event.screenY
  }

  isDragging.value = true
  // 启动拖拽时的高频位置更新
  startDragTracking()
  
  document.addEventListener('pointermove', handlePointerMove)
  document.addEventListener('pointerup', handlePointerUp)
  isHovering.value = false
}

const handlePointerMove = async (event) => {  
  console.log('删除点击定时器')
  clearTimeout(clickPetTimeout)

  // 检查鼠标是否仍然处于按下状态
  if (event.buttons === 0) {
    console.log('鼠标已释放，但move事件仍被触发，立即清理监听器')
    cleanupEventListeners()
    return
  }

  const deltaX = event.screenX - dragStartPos.value.x
  const deltaY = event.screenY - dragStartPos.value.y
  
  const newX = windowStartPos.value.x + deltaX
  const newY = windowStartPos.value.y + deltaY
  
  try {
    await currentWindow.setPosition(new LogicalPosition(newX, newY))
    // 拖拽时实时更新位置（通过 dragUpdateInterval 处理）
  } catch (error) {
    console.error('移动窗口失败:', error)
  }

  // 禁止点击 500ms
  allowClickPet.value = false
  clickPetTimeout = setTimeout(async () => {
    allowClickPet.value = true
  }, 500)
}

const handlePointerUp = async () => {
  isDragging.value = false
  // 停止拖拽跟踪
  stopDragTracking()
  cleanupEventListeners()
  
  // 拖拽结束时确保位置更新
  await updateWindowPosition()
  if (hasMenuWindow.value) {
    await updateMenuWindowPosition()
  }
}

// 鼠标进入桌宠区域
const handlePointerEnter = (event) => {
  isHovering.value = true
}

// 鼠标离开桌宠区域
const handlePointerLeave = (event) => {
  isHovering.value = false
}

// 左键切换菜单窗口
const handleLeftClick = async (event) => {
  if (!allowClickPet.value) {
    console.log('点击被禁止')
    return
  }

  console.log('🖱️ 桌宠被点击，切换菜单窗口')

  try {
    const result = await toggleMenuWindow()
    // 更新菜单窗口状态
    hasMenuWindow.value = checkMenuWindowExists()
    
    if (hasMenuWindow.value) {
      console.log('📋 菜单窗口已打开')
      // 确保菜单窗口位置正确
      await updateWindowPosition()
      await updateMenuWindowPosition()
    } else {
      console.log('📋 菜单窗口已关闭')
    }
  } catch (error) {
    console.error('切换菜单窗口失败:', error)
  }
}

// 右键显示菜单
const handleContextMenu = (event) => {
  event.preventDefault()
  event.stopPropagation()
  console.log('右键菜单')
  
  const rect = event.currentTarget.getBoundingClientRect()
  const menuPosition = {
    x: rect.right + 10,
    y: Math.max(10, rect.top)
  }
}

// 清除全局监听
const cleanupEventListeners = () => {
  document.removeEventListener('pointermove', handlePointerMove)
  document.removeEventListener('pointerup', handlePointerUp)
}
</script>

<template>
  <div
    class="desktop-pet"
    :style="{
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
        :class="['pet-image', { 'hover': isHovering, 'has-window': hasMenuWindow }]"
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
  top: 10px;
  left: 10px;
  background: transparent;
  position: relative;
}

.pet-image {
  width: 130px;
  height: 75px;
  filter: drop-shadow(2px 2px 4px rgba(0, 0, 0, 0.3));
  transition: all 0.3s ease;
  background: transparent;
  flex-shrink: 0;
}

.pet-image.hover {
  transform: scale(1.1);
}

.pet-image.has-window {
  filter: drop-shadow(0 0 8px rgba(74, 144, 226, 0.6));
}
</style>