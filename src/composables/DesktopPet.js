import { ref, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window'
import { 
  windowInstances, 
  toggleClipboardWindow, 
  updateMainWindowPosition, 
  toggleMenuWindow,
  updateMenuWindowPosition,
  updateMenuWindowPositionRealTime,
  hasMenuWindow as checkMenuWindowExists
} from '../utils/actions.js'
import { 
  AnimationManager, 
  AnimationState, 
  getAnimationForKey, 
  getAnimationForMouse 
} from './utils/animations.js'

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
  const animationFrame = ref('idle_1') // 当前动画帧

  // 初始化动画管理器
  const animationManager = new AnimationManager()

  let clickPetTimeout = null
  let positionUpdateInterval = null
  let dragUpdateInterval = null

  // 启动位置跟踪（常规更新）
  const startPositionTracking = () => {
    positionUpdateInterval = setInterval(async () => {
      if (hasMenuWindow.value && !isDragging.value) {
        await updateWindowPosition()
        await updateMenuWindowPosition()
      }
    }, 500)
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
        await updateMenuWindowPositionRealTime()
      }
    }, 50)
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
    startDragTracking()
    
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
    stopDragTracking()
    cleanupEventListeners()
    
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
      hasMenuWindow.value = checkMenuWindowExists()
      
      if (hasMenuWindow.value) {
        console.log('📋 菜单窗口已打开')
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

  // 设置动画回调
  const setupAnimationCallbacks = () => {
    animationManager.on('onFrameChange', (state, frameIndex) => {
      const config = ANIMATION_CONFIG[state]
      if (config && config.frames[frameIndex]) {
        animationFrame.value = config.frames[frameIndex]
      }
    })

    animationManager.on('onStateChange', (oldState, newState) => {
      console.log(`动画状态: ${oldState} → ${newState}`)
    })
  }

  // 监听全局键盘事件
  const setupGlobalListeners = async () => {
    try {
      // 监听键盘按下事件
      await listen('key-down', (event) => {
        console.log('键盘按下:', event.payload)
        handleKeyPress(event.payload)
      })

      // 监听键盘释放事件
      await listen('key-up', (event) => {
        // 可以在这里处理键盘释放的动画
        console.log('键盘释放:', event.payload)
      })

      // 监听全局鼠标点击事件
      await listen('global-mouse-down', (event) => {
        handleGlobalMouseDown(event.payload)
      })

      // 监听全局鼠标释放事件
      await listen('global-mouse-up', (event) => {
        handleGlobalMouseUp(event.payload)
      })

    } catch (error) {
      console.error('设置全局监听器失败:', error)
    }
  }

  // 处理键盘按下
  const handleKeyPress = (keyEvent) => {
    if (!keyEvent || !keyEvent.code) return
    
    const animationType = getAnimationForKey(keyEvent.code)
    
    // 根据按键类型触发不同的动画
    switch(animationType) {
      case 'left_paw':
        animationManager.setState(AnimationState.LEFT_CLICK)
        break
      case 'right_paw':
        animationManager.setState(AnimationState.RIGHT_CLICK)
        break
      case 'both_paws':
        // 双爪动画
        animationManager.setState(AnimationState.KEY_PRESS)
        break
      default:
        animationManager.setState(AnimationState.KEY_PRESS)
    }
    
    // 动画持续时间后返回空闲状态
    setTimeout(() => {
      if (animationManager.currentState !== AnimationState.IDLE && 
          !animationManager.isAnimating) {
        animationManager.setState(AnimationState.IDLE)
      }
    }, 300)
  }

  // 处理全局鼠标按下
  const handleGlobalMouseDown = (mouseEvent) => {
    if (!mouseEvent || !mouseEvent.button) return
    
    const button = mouseEvent.button === 0 ? 'left' : 
                   mouseEvent.button === 1 ? 'middle' : 'right'
    
    const animationState = getAnimationForMouse(button)
    animationManager.setState(animationState)
  }

  // 处理全局鼠标释放
  const handleGlobalMouseUp = (mouseEvent) => {
    // 鼠标释放后，如果不是正在动画，返回空闲状态
    if (!animationManager.isAnimating) {
      setTimeout(() => {
        animationManager.setState(AnimationState.IDLE)
      }, 100)
    }
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
      updateMainWindowPosition(currentPosition.value, { width: 120, height: 120 })
      
      // 初始化动画系统
      setupAnimationCallbacks()
      animationManager.setState(AnimationState.IDLE)
      
      // 设置全局事件监听
      await setupGlobalListeners()

      // 启动位置跟踪
      startPositionTracking()
    } catch (error) {
      console.error('设置窗口大小失败:', error)
    }
  })

  onUnmounted(() => {
    stopPositionTracking()
    stopDragTracking()
    cleanupEventListeners()
    animationManager.destroy()
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
    handleContextMenu,
    animationFrame
  }
}