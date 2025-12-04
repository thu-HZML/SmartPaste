import { ref, computed, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { 
  windowInstances, 
  updateMainWindowPosition, 
  toggleMenuWindow,
  updateMenuWindowPosition,
  hasMenuWindow as checkMenuWindowExists
} from '../utils/actions.js'
import { 
  AnimationManager, 
  AnimationState, 
  getAnimationForKey, 
  getAnimationForMouse,
  ANIMATION_CONFIG
} from '../utils/animations.js'

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
  const animationFrame = ref('cover') // 当前动画帧

  // 初始化动画管理器
  const animationManager = new AnimationManager()

  let clickPetTimeout = null
  let positionUpdateInterval = null
  let dragUpdateInterval = null

  // 根据动画帧计算图片路径
  const petImagePath = computed(() => {
    return `/resources/${animationFrame.value}.png`
  })

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

  // 设置动画回调 - 修复帧更新逻辑
  const setupAnimationCallbacks = () => {
    animationManager.on('onFrameChange', (state, frameIndex) => {
      
      // 获取对应状态的配置
      const config = ANIMATION_CONFIG[state]
      if (config && config.frames && config.frames.length > 0) {
        // 确保帧索引在有效范围内
        const safeFrameIndex = frameIndex % config.frames.length
        const newFrame = config.frames[safeFrameIndex]
        
        console.log('新动画帧:', newFrame)
        animationFrame.value = newFrame
      }
    })

    animationManager.on('onStateChange', (oldState, newState) => {
      console.log(`动画状态变化: ${oldState} → ${newState}`)
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
      animationManager.setState(AnimationState.IDLE, true)
      setupAnimationCallbacks()    

      // 设置全局事件监听
      await setupGlobalListeners()

      // 启动位置跟踪
      startPositionTracking()
      await updateMainWindowPosition(currentPosition.value)
    } catch (error) {
      console.error('设置窗口大小失败:', error)
    }
  })

  onUnmounted(() => {
    cleanupEventListeners()
    animationManager.destroy()
  })

  return {
    // 响应式状态
    isHovering,
    hasClipboardWindow,
    hasMenuWindow,
    isDragging,

    // 计算属性
    petImagePath,

    // 事件处理函数
    handlePointerEnter,
    handlePointerLeave,
    handlePointerDown,
    handleLeftClick,
    handleContextMenu,
    animationFrame
  }
}