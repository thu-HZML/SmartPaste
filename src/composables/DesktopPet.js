import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window'
import { 
  windowInstances, 
  updateMainWindowPosition, 
  toggleMenuWindow,
  updateMenuWindowPosition,
  hasMenuWindow as checkMenuWindowExists
} from '../utils/actions.js'

export function useDesktopPet() {
  const isHovering = ref(false)
  const hasClipboardWindow = ref(false)
  const hasMenuWindow = ref(false)
  const isDragging = ref(false)
  const dragStartPos = ref({ x: 0, y: 0 })
  const windowStartPos = ref({ x: 0, y: 0 })
  const currentWindow = getCurrentWindow()
  const scaleFactor = ref(1.486)
  const allowClickPet = ref(true)
  const currentPosition = ref({ x: 0, y: 0 })

  let clickPetTimeout = null
  let positionUpdateInterval = null
  let dragUpdateInterval = null

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
    
    document.addEventListener('pointermove', handlePointerMove)
    document.addEventListener('pointerup', handlePointerUp)
    isHovering.value = false
  }

  const handlePointerMove = async (event) => {  
    console.log('删除点击定时器')
    clearTimeout(clickPetTimeout)

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
      currentPosition.value = { x: newX, y: newY }
      await updateMainWindowPosition(currentPosition.value)
      await updateMenuWindowPosition()
    } catch (error) {
      console.error('移动窗口失败:', error)
    }

    allowClickPet.value = false
    clickPetTimeout = setTimeout(async () => {
      allowClickPet.value = true
    }, 500)
  }

  const handlePointerUp = async () => {
    isDragging.value = false
    cleanupEventListeners()
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
      hasMenuWindow.value = checkMenuWindowExists()
      
      if (hasMenuWindow.value) {
        console.log('📋 菜单窗口已打开')
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

  onMounted(async () => {
    console.log('[DesktopPet] mounted')
    try {
      await currentWindow.setSize(new LogicalSize(150, 95))
      await currentWindow.setPosition(new LogicalPosition(1550, 800))
      const actualScaleFactor = await currentWindow.scaleFactor()
      console.log('系统缩放比例:', actualScaleFactor)
      scaleFactor.value = actualScaleFactor
      
      const position = await currentWindow.outerPosition()
      currentPosition.value = {
        x: Math.round(position.x / scaleFactor.value),
        y: Math.round(position.y / scaleFactor.value)
      }
      await updateMainWindowPosition(currentPosition.value)
    } catch (error) {
      console.error('设置窗口大小失败:', error)
    }
  })

  onUnmounted(() => {
    cleanupEventListeners()
  })

  return {
    isHovering,
    hasClipboardWindow,
    hasMenuWindow,
    isDragging,
    handlePointerEnter,
    handlePointerLeave,
    handlePointerDown,
    handleLeftClick,
    handleContextMenu
  }
}