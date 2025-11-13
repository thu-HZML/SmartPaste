<script setup>
import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
import { toggleClipboardWindow, updateMainWindowPosition } from '../utils/actions.js'

const isHovering = ref(false)
const hasClipboardWindow = ref(false)
const isDragging = ref(false)
const dragStartPos = ref({ x: 0, y: 0 })
const windowStartPos = ref({ x: 0, y: 0 })
const currentWindow = getCurrentWindow();
const scaleFactor = ref(1.486) // 根据调试信息计算的缩放比例
const allowClickPet = ref(true)

const emit = defineEmits(['show-menu', 'hide-menu'])

// 点击防抖定时器
let clickPetTimeout = null

onMounted(async () => {
  console.log('[DesktopPet] mounted')

  try {
    await currentWindow.setSize(new LogicalSize(100, 100));
    await currentWindow.setPosition(new LogicalPosition(1600, 800))
    const actualScaleFactor = await currentWindow.scaleFactor();
    console.log('系统缩放比例:', actualScaleFactor);
    scaleFactor.value = actualScaleFactor;
  } catch (error) {
    console.error('设置窗口大小失败:', error)
  }
})

// 鼠标按下桌宠 - 开始拖动
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
  
  // 记录鼠标按下时的屏幕坐标
  dragStartPos.value = {
    x: event.screenX,
    y: event.screenY
  }

  // 添加全局事件监听
  document.addEventListener('pointermove', handlePointerMove)
  document.addEventListener('pointerup', handlePointerUp)
  isHovering.value = false
}

// 鼠标移动 - 处理拖动
const handlePointerMove = async (event) => {  
  console.log('删除点击定时器')
  clearTimeout(clickPetTimeout)

  const deltaX = event.screenX - dragStartPos.value.x
  const deltaY = event.screenY - dragStartPos.value.y
  
  
  // 更新窗口位置
  const newX = windowStartPos.value.x + deltaX
  const newY = windowStartPos.value.y + deltaY
  
  try {
    await currentWindow.setPosition(new LogicalPosition(newX, newY))
    const position = await currentWindow.outerPosition()
  } catch (error) {
    console.error('移动窗口失败:', error)
  }

  // 禁止点击 20ms
  allowClickPet.value = false
  console.log('设置点击定时器')
  clickPetTimeout = setTimeout(async () => {
    allowClickPet.value = true
  }, 500)
}

// 鼠标释放 - 结束拖动
const handlePointerUp = () => {
  isDragging.value = false
  cleanupEventListeners()
}

// 鼠标进入桌宠区域
const handlePointerEnter = (event) => {
  isHovering.value = true
  console.log('鼠标进入，isHovering:', isHovering.value)
}

// 鼠标离开桌宠区域
const handlePointerLeave = (event) => {
  isHovering.value = false
  console.log('鼠标离开，isHovering:', isHovering.value)
}

// 左键切换剪贴板窗口
const handleLeftClick = async (event) => {
  if (!allowClickPet.value) {
    console.log('点击被禁止')
    return
  }

  console.log('🖱️ 桌宠被点击，切换剪贴板窗口')

  setTimeout(() => {
    handlePointerUp()
  }, 10)

  try {
    const result = await toggleClipboardWindow()
    hasClipboardWindow.value = !hasClipboardWindow.value
    
    if (hasClipboardWindow.value) {
      console.log('📋 剪贴板窗口已打开')
    } else {
      console.log('📋 剪贴板窗口已关闭')
    }
  } catch (error) {
    console.error('切换剪贴板窗口失败:', error)
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

  emit('show-menu', menuPosition)
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
        :class="['pet-image', { 'hover': isHovering, 'has-window': hasClipboardWindow }]"
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
  background: transparent;
  position: relative;
}

.pet-image {
  width: 100px;
  height: 100px;
  filter: drop-shadow(2px 2px 4px rgba(0, 0, 0, 0.3));
  transition: all 0.3s ease;
  background: transparent;
}

.pet-image.hover {
  transform: scale(1.1);
}

.pet-image.has-window {
  filter: drop-shadow(0 0 8px rgba(74, 144, 226, 0.6));
}
</style>