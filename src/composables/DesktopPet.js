import { ref, computed, onMounted, onUnmounted } from 'vue'
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
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
  getAnimationForMouse,
} from '../utils/animations.js'
import live2d from '../utils/live2dManager.js'

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
  const animationFrame = ref('background') // 当前动画帧
  const currentKey = ref('') // 当前按下的按键
  const currentAnimationState = ref(AnimationState.IDLE)

  // 可用按键集合
  const availableKeyImages = new Set([
    'Alt', 'AltGr', 'BackQuote', 'Backspace', 'CapsLock', 'Control', 
    'ControlLeft', 'ControlRight', 'Delete', 'Escape', 'Fn', 'KeyA', 
    'KeyB', 'KeyC', 'KeyD', 'KeyE', 'KeyF', 'KeyG', 'KeyH', 'KeyI', 
    'KeyJ', 'KeyK', 'KeyL', 'KeyM', 'KeyN', 'KeyO', 'KeyP', 'KeyQ', 
    'KeyR', 'KeyS', 'KeyT', 'KeyU', 'KeyV', 'KeyW', 'KeyX', 'KeyY', 
    'KeyZ', 'Meta', 'Num0', 'Num1', 'Num2', 'Num3', 'Num4', 'Num5', 
    'Num6', 'Num7', 'Num8', 'Num9', 'Return', 'Shift', 'ShiftLeft', 
    'ShiftRight', 'Slash', 'Space', 'Tab'
  ])

  // 初始化动画管理器
  const animationManager = new AnimationManager()

  let clickPetTimeout = null

  // 根据动画帧计算图片路径
  const petImagePath = computed(() => {
    const state = currentAnimationState.value
    // 按键状态：使用按键对应的图片
    if (state === AnimationState.KEY_PRESS) {
      const keyImage = currentKey.value || 'key'
      return `/resources/left-keys/${keyImage}.png`
    }
    return `/resources/${animationFrame.value}.png`
  })

  // 根据动画帧计算背景图片路径
  const petBackgroundPath = computed(() => {
    return `/resources/background.png`
  })

  // 根据动画状态计算是否显示动画层
  const showPetAnimation = computed(() => {
    const state = currentAnimationState.value

    // 待机状态：不显示动画层
    if (state === AnimationState.IDLE) {
      return false
    }
    return true
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
      updateMainWindowPosition(currentPosition.value)
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
      const currentFrame = animationManager.getCurrentFrame()
      console.log('动画帧更新:', state, '->', currentFrame)
      animationFrame.value = currentFrame
    })

    animationManager.on('onStateChange', (oldState, newState) => {
      console.log(`动画状态变化: ${oldState} → ${newState}`)
      
      currentAnimationState.value = newState

      // 如果从按键状态切换到其他状态，清空当前按键
      if (oldState === AnimationState.KEY_PRESS) {
        currentKey.value = null
      }
    })
  }

  // 监听全局键盘事件
  const setupGlobalListeners = async () => {
    try {
      // 开启全局键盘监听
      await invoke('start_key_listener');

      // 监听键盘事件
      await listen('key-monitor-event', (event) => {
        const data = event.payload;
        if (data.type === 'down') {
          handleKeyPress(data.key)
        } else if (data.type === 'up') {
          handleKeyUp(data.key)
        }
      });

      /*
      // 监听全局鼠标点击事件
      await listen('global-mouse-down', (event) => {
        handleGlobalMouseDown(event.payload)
      })

      // 监听全局鼠标释放事件
      await listen('global-mouse-up', (event) => {
        handleGlobalMouseUp(event.payload)
      })
        */

    } catch (error) {
      console.error('设置全局监听器失败:', error)
    }
  }

  // 处理键盘按下
  const handleKeyPress = (key) => {
    if (!availableKeyImages.has(key)) {
      console.log('按键不在图片列表中，显示默认 Enter 键')
      key = 'Return'
    }
    currentKey.value = key
    // 设置按键动画状态，并传递自定义帧
    animationManager.setState(AnimationState.KEY_PRESS, [key])
  }

  const handleKeyUp = (key) => {
    // 如果是按键状态，返回空闲状态
    if (animationManager.currentState === AnimationState.KEY_PRESS) {
      // 可以设置一个延迟，让按键图片显示一段时间
      animationManager.setState(AnimationState.IDLE)
    }
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

  // 在 DesktopPet.js 的 initLive2D 函数中
  const initLive2D = async () => {
    try {
      console.log('开始加载模型...')
      
      // 使用绝对路径 - 之前成功过的路径
      const modelPath = 'C:/Users/heyufei/Desktop/bigHW/SmartPaste/public/resources/live2d'
      console.log('使用路径:', modelPath)
      
      const result = await live2d.load(modelPath)
      
      // 初始调整大小
      setTimeout(() => {
        live2d.resizeModel()
      }, 100)
      
      console.log('模型加载成功', result)
    } catch (err) {
      console.error('加载模型失败:', err)
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

      updateMainWindowPosition(currentPosition.value)

      // 初始化 Live2D
      await initLive2D()
    } catch (error) {
      console.error('设置窗口大小失败:', error)
    }
  })

  onUnmounted(async () => {
    cleanupEventListeners()
    animationManager.destroy()
    await invoke('stop_key_listener');
  })

  return {
    // 响应式状态
    isHovering,
    hasClipboardWindow,
    hasMenuWindow,
    isDragging,

    // 计算属性
    petImagePath,
    petBackgroundPath,
    showPetAnimation,

    // 事件处理函数
    handlePointerEnter,
    handlePointerLeave,
    handlePointerDown,
    handleLeftClick,
    handleContextMenu,
    animationFrame
  }
}