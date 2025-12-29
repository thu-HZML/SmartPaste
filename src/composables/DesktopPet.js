import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/core'
import { 
  updateMainWindowPosition, 
  toggleMenuWindow,
  updateMenuWindowPosition,
  toggleAiWindow,
  updateAiWindowPosition,
  hasMenuWindow as checkMenuWindowExists,
  updateScreenWorkArea
} from '../utils/actions.js'
import { 
  AnimationManager, 
  AnimationState, 
} from '../utils/animations.js'
import live2d from '../utils/live2dManager.js'
import { useSettingsStore } from '../stores/settings'
import { executeCloudPush } from './Preferences'

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
  const settings = useSettingsStore().settings

  // 全局监听器
  let unlistenKeyButton = null
  let unlistenMouseButton = null
  let unlistenMouseMove = null
  let unlistenAiEnabledChanged = null

  // 添加剪贴板监听器的取消函数引用
  const unlistenClipboardUpdated = ref(null)

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

  //云端同步相关
  const FREQUENCY_MAP = {
    'realtime': 30 * 1000,     // 实时：30秒
    '5min': 5 * 60 * 1000,     // 5分钟
    '15min': 15 * 60 * 1000,   // 15分钟
    '1hour': 60 * 60 * 1000    // 1小时
  }

  const IDLE_CHECK_INTERVAL = 30 * 1000;
  let syncTimer = null

  const executeSyncLoop = async () => {
    console.log('🔄 [SyncLoop] 正在执行同步循环检查...')
    let currentConfig = {}
    let shouldSync = false
    let nextDelay = IDLE_CHECK_INTERVAL

    try {
      // 尝试从后端读取 config.json
      const configStr = await invoke('get_config_json')
      if (configStr) {
        currentConfig = JSON.parse(configStr)
        // console.log('📂 [SyncLoop] 读取到后端配置:', currentConfig.sync_frequency, currentConfig.cloud_sync_enabled)
      } else {
        console.warn('⚠️ [SyncLoop] 后端返回配置为空')
        currentConfig = settings // 降级使用内存配置
      }

      // 决定是否同步
      if (currentConfig.cloud_sync_enabled) {
        shouldSync = true
        // 计算下一次正常同步的时间
        const freq = currentConfig.sync_frequency || '5min'
        nextDelay = FREQUENCY_MAP[freq] || FREQUENCY_MAP['5min']
      } else {
        console.log('⏸️ [SyncLoop] 同步功能已禁用 (将进入待机轮询模式)')
        // 如果被禁用，不停止循环，而是用较慢的速度轮询配置，等待它变回 true
        nextDelay = IDLE_CHECK_INTERVAL 
      }

    } catch (e) {
      console.error('❌ [SyncLoop] 读取配置失败:', e)
      nextDelay = 60 * 1000
    }

    // 同步
    if (shouldSync) {
      try {
        console.log('🚀 [SyncLoop] 开始执行上传...')
        await executeCloudPush()
        console.log(`✅ [SyncLoop] 同步成功! 下次同步: ${nextDelay/1000}秒后`)
      } catch (e) {
        console.error('❌ [SyncLoop] 上传过程出错:', e)
      }
    } else {
      console.log(`💤 [SyncLoop] 跳过本次上传. 下次检查: ${nextDelay/1000}秒后`)
    }
    // 重新设置定时器
    if (syncTimer !== null) { // 确保没有被 unmount 清除
        clearTimeout(syncTimer) 
        syncTimer = setTimeout(executeSyncLoop, nextDelay)
    }
  }

  // 启动入口
  const startSyncTimer = () => {
    // 防止重复启动
    if (syncTimer) {
      console.log('⚡ [SyncLoop] 定时器已存在，重置中...')
      clearTimeout(syncTimer)
    }
    
    // 初始化 timer 占位符，防止 executeSyncLoop 里的判断失效
    syncTimer = 1 

    setTimeout(executeSyncLoop, 1000)
  }

  watch(
    () => [settings.ai_enabled, settings.cloud_sync_enabled, settings.sync_frequency],
    ([newAi, newSync, newFreq], [oldAi, oldSync, oldFreq]) => {
      // AI 监听逻辑
      if (newAi !== oldAi) {
        console.log(`AI功能设置变化: ${oldAi} -> ${newAi}`)
        setupClipboardRelay()
      }
      
      // 同步 监听逻辑
      if (newSync !== oldSync || newFreq !== oldFreq) {
        console.log('检测到同步设置变更，重启定时器...')
        startSyncTimer()
      }
    }
  )

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
      await updateAiWindowPosition()
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

  // 监听全局事件
  const setupGlobalListeners = async () => {
    try {
      // 开启后端剪贴板监听
      await setupClipboardRelay()

      // 开启全局监听（键盘点击、鼠标点击、鼠标移动）
      await invoke('start_key_listener');
      await invoke('start_mouse_button_listener');
      await invoke('start_mouse_move_listener');

      // 监听键盘事件
      unlistenKeyButton = await listen('key-monitor-event', (event) => {
        const data = event.payload;
        if (data.type === 'down') {
          handleKeyPress(data.key)
        } else if (data.type === 'up') {
          handleKeyUp(data.key)
        }
      });

      
      // 监听全局鼠标点击事件
      unlistenMouseButton = await listen('mouse-button-event', (event) => {
        const { button, type } = event.payload;
        if (type === 'down') {
          handleGlobalMouseDown(button)
        } else if (type === 'up') {
          handleGlobalMouseUp(button)
        }
      })

      unlistenMouseMove = await listen('mouse-move-event', (event) => {
        const { x, y, raw_x, raw_y } = event.payload;
        handleGlobalMouseMove( x, y )
      })

      // 监听 AI 设置变更事件
      unlistenAiEnabledChanged = await listen('ai-enabled-changed', (event) => {
        const { enabled } = event.payload
        console.log(`📡 收到 ai_enabled 变更事件: ${enabled}`)
        
        // 直接更新 settings 的值
        settings.ai_enabled = enabled
      })
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

    live2d.setParameterValue("CatParamLeftHandDown", 1)

    // 设置按键动画状态，并传递自定义帧
    animationManager.setState(AnimationState.KEY_PRESS, [key])

  }

  const handleKeyUp = (key) => {
    // 如果是按键状态，返回空闲状态
    if (animationManager.currentState === AnimationState.KEY_PRESS) {
      live2d.setParameterValue("CatParamLeftHandDown", 0)
      animationManager.setState(AnimationState.IDLE)
    }
  }

  // 处理全局鼠标按下
  const handleGlobalMouseDown = (mouseButton) => {   
    if (mouseButton === 'left') {   
      live2d.setParameterValue("ParamMouseLeftDown", 1)
    } else if (mouseButton === 'right') {
      live2d.setParameterValue("ParamMouseRightDown", 1)
    }
  }

  // 处理全局鼠标释放
  const handleGlobalMouseUp = (mouseButton) => {
    if (mouseButton === 'left') {   
      live2d.setParameterValue("ParamMouseLeftDown", 0)
    } else if (mouseButton === 'right') {
      live2d.setParameterValue("ParamMouseRightDown", 0)
    }
  }

  // 处理全局鼠标移动
  const handleGlobalMouseMove = ( x, y ) => {
    const realx = ( x - 0.5 ) * (-60)
    const realy = ( y - 0.5 ) * 60
    live2d.setParameterValue("ParamMouseX", realx)
    live2d.setParameterValue("ParamAngleX", -realx)
    live2d.setParameterValue("ParamMouseY", realy)
    live2d.setParameterValue("ParamAngleY", realy)
  }

  // 在 DesktopPet.js 的 initLive2D 函数中
  const initLive2D = async () => {
    try {
      console.log('开始加载模型...')

      // 获取 utils 目录路径
      const utilsDirPath = await invoke('get_utils_dir_path');

      // 替换成live2d资源在的绝对路径
      const modelPath = utilsDirPath.replace('//?/', '').replace('/src-tauri/src', '/src-tauri') + '/resources/live2d'
      console.log('使用路径:', modelPath)
      
      const result = await live2d.load(modelPath)
      
      // 初始调整大小
      live2d.resizeModel()
      
      console.log('模型加载成功', result)
    } catch (err) {
      console.error('加载模型失败:', err)
    }
  }

  // 主窗口监听剪贴板事件
  const setupClipboardRelay = async () => {
    // 先移除现有的监听器
    if (unlistenClipboardUpdated.value) {
      unlistenClipboardUpdated.value()
      unlistenClipboardUpdated.value = null
    }

    // 只有当ai_enabled为true时才设置监听器
    if (settings.ai_enabled) {
      console.log('AI功能已启用，设置剪贴板监听器')
      const unlisten = await listen('clipboard-updated', async (event) => {
        console.log('接受后端更新消息')

        // 打开AI窗口
        await toggleAiWindow()
      })
      
      unlistenClipboardUpdated.value = unlisten
      console.log('剪贴板监听器已设置')
    } else {
      console.log('AI功能已禁用，不设置剪贴板监听器')
    }
  }

  // 移除剪贴板监听器
  const removeClipboardRelay = () => {
    if (unlistenClipboardUpdated.value) {
      unlistenClipboardUpdated.value()
      unlistenClipboardUpdated.value = null
      console.log('剪贴板监听器已移除')
    }
  }

  onMounted(async () => {
    console.log('[DesktopPet] mounted')
    try {
      await currentWindow.setSize(new LogicalSize(150, 95))

      // 获取实际缩放比例
      const actualScaleFactor = await currentWindow.scaleFactor()
      console.log('系统缩放比例:', actualScaleFactor)
      scaleFactor.value = actualScaleFactor

      // 获取屏幕分辨率
      const [width, height] = await invoke('get_screen_resolution')
      console.log(`屏幕分辨率: ${width}x${height}`)
      const windowSize = {
        width: width / actualScaleFactor,
        height: height / actualScaleFactor,
      }
      console.log(`屏幕分辨率: `, windowSize)
      localStorage.setItem('windowSize', JSON.stringify(windowSize))

      // 初始位置放在右下角
      await currentWindow.setPosition(new LogicalPosition(windowSize.width - 150, windowSize.height - 165))
      
      
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

      // 组件挂载时启动同步定时器
      startSyncTimer()

      updateMainWindowPosition(currentPosition.value)

      // 初始化 Live2D
      await initLive2D()
      
      await updateScreenWorkArea()
    } catch (error) {
      console.error('设置窗口大小失败:', error)
    }
  })

  onUnmounted(async () => {
    cleanupEventListeners()
    animationManager.destroy()

    if (syncTimer) {
      clearTimeout(syncTimer)
      syncTimer = null
    }

    // 停止全局监听
    await invoke('stop_key_listener');
    await invoke('stop_mouse_listener');

    unlistenKeyButton()
    unlistenMouseButton()
    unlistenMouseMove()
    unlistenAiEnabledChanged()

    removeClipboardRelay()
  })

  return {
    // 响应式状态
    isHovering,
    hasClipboardWindow,
    hasMenuWindow,
    isDragging,
    unlistenClipboardUpdated,

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
    animationFrame,
    setupClipboardRelay,
    removeClipboardRelay
  }
}