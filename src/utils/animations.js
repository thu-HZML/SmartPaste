// 动画状态定义
export const AnimationState = {
  IDLE: 'idle',
  LEFT_CLICK: 'left_click',
  RIGHT_CLICK: 'right_click',
  KEY_PRESS: 'key_press',
  DRAGGING: 'dragging',
  HOVER: 'hover'
}

// 动画配置
export const ANIMATION_CONFIG = {
  [AnimationState.IDLE]: {
    duration: 1000, // 毫秒
    loop: true,
    frames: ['cover']
  },
  [AnimationState.LEFT_CLICK]: {
    duration: 300,
    loop: false,
    frames: ['left_down', 'left_mid', 'left_up'],
    returnTo: AnimationState.IDLE
  },
  [AnimationState.RIGHT_CLICK]: {
    duration: 300,
    loop: false,
    frames: ['right_down', 'right_mid', 'right_up'],
    returnTo: AnimationState.IDLE
  },
  [AnimationState.KEY_PRESS]: {
    duration: 200,
    loop: false,
    frames: ['key_press_1', 'key_press_2', 'key_press_3'],
    returnTo: AnimationState.IDLE
  },
  [AnimationState.DRAGGING]: {
    duration: 500,
    loop: true,
    frames: ['drag_1', 'drag_2']
  },
  [AnimationState.HOVER]: {
    duration: 400,
    loop: false,
    frames: ['hover_1', 'hover_2'],
    returnTo: AnimationState.IDLE
  }
}

// 动画管理器
export class AnimationManager {
  constructor() {
    this.currentState = AnimationState.IDLE
    this.currentFrame = 0
    this.animationTimer = null
    this.isAnimating = false
    this.callbacks = {
      onFrameChange: null,
      onStateChange: null
    }
  }

  // 切换动画状态
  setState(newState, force = false) {
    if (this.currentState === newState && !force) return
    
    const oldState = this.currentState
    this.currentState = newState
    this.currentFrame = 0
    this.isAnimating = true
    
    // 停止之前的动画
    if (this.animationTimer) {
      clearTimeout(this.animationTimer)
    }
    
    // 触发状态改变回调
    if (this.callbacks.onStateChange) {
      this.callbacks.onStateChange(oldState, newState)
    }
    
    // 开始新动画
    this.startAnimation()
  }

  // 开始播放动画
  startAnimation() {
    const config = ANIMATION_CONFIG[this.currentState]
    if (!config || config.frames.length === 0) return

    const frameCount = config.frames.length
    const frameDuration = config.duration / frameCount
    
    const animate = () => {
      // 触发帧改变回调
      if (this.callbacks.onFrameChange) {
        this.callbacks.onFrameChange(this.currentState, this.currentFrame)
      }
      
      this.currentFrame++
      
      // 检查动画是否结束
      if (this.currentFrame >= frameCount) {
        if (config.loop) {
          this.currentFrame = 0
          this.animationTimer = setTimeout(animate, frameDuration)
        } else {
          this.isAnimating = false
          // 返回到指定状态
          if (config.returnTo) {
            setTimeout(() => {
              this.setState(config.returnTo)
            }, 100)
          }
        }
      } else {
        this.animationTimer = setTimeout(animate, frameDuration)
      }
    }
    
    // 开始第一帧
    animate()
  }

  // 获取当前动画帧
  getCurrentFrame() {
    const config = ANIMATION_CONFIG[this.currentState]
    if (!config || config.frames.length === 0) return ''
    
    const frameIndex = this.currentFrame % config.frames.length
    return config.frames[frameIndex]
  }

  // 注册回调函数
  on(event, callback) {
    if (this.callbacks.hasOwnProperty(event)) {
      this.callbacks[event] = callback
    }
  }

  // 清理
  destroy() {
    if (this.animationTimer) {
      clearTimeout(this.animationTimer)
    }
    this.callbacks = {}
  }
}

// 键盘映射到动画
export const KEY_ANIMATION_MAP = {
  // 主键盘区
  'KeyA': 'left_paw',
  'KeyS': 'left_paw',
  'KeyD': 'right_paw',
  'KeyF': 'right_paw',
  'Space': 'both_paws',
  // 数字键
  'Digit1': 'left_paw',
  'Digit2': 'left_paw',
  'Digit3': 'right_paw',
  'Digit4': 'right_paw',
  // 方向键
  'ArrowLeft': 'left_paw',
  'ArrowRight': 'right_paw',
  'ArrowUp': 'both_paws',
  'ArrowDown': 'both_paws',
  // 功能键
  'Enter': 'both_paws',
  'Tab': 'right_paw'
}

// 鼠标事件映射
export const MOUSE_ANIMATION_MAP = {
  'left': AnimationState.LEFT_CLICK,
  'right': AnimationState.RIGHT_CLICK,
  'middle': AnimationState.KEY_PRESS
}

// 获取对应的动画类型
export function getAnimationForKey(keyCode) {
  return KEY_ANIMATION_MAP[keyCode] || 'key_press'
}

export function getAnimationForMouse(button) {
  return MOUSE_ANIMATION_MAP[button] || AnimationState.KEY_PRESS
}