// 动画状态定义
export const AnimationState = {
  IDLE: 'idle',
  LEFT_CLICK: 'left_click',
  RIGHT_CLICK: 'right_click',
  KEY_PRESS: 'key_press',  // 专门用于键盘按键
  DRAGGING: 'dragging',
  HOVER: 'hover'
}

// 动态动画配置
export const getAnimationConfig = () => ({
  [AnimationState.IDLE]: {
    duration: 1000,
    loop: true,
    animate: true,
    frames: ['cover']
  },
  [AnimationState.LEFT_CLICK]: {
    duration: 300,
    loop: false,
    animate: true,
    frames: ['left_down', 'left_mid', 'left_up'],
    returnTo: AnimationState.IDLE
  },
  [AnimationState.RIGHT_CLICK]: {
    duration: 300,
    loop: false,
    animate: true,
    frames: ['right_down', 'right_mid', 'right_up'],
    returnTo: AnimationState.IDLE
  },
  [AnimationState.KEY_PRESS]: {
    duration: 300,
    loop: false,
    animate: false,
    frames: [], // 动态设置
    returnTo: AnimationState.IDLE
  },
  [AnimationState.DRAGGING]: {
    duration: 500,
    loop: true,
    animate: true,
    frames: ['drag_1', 'drag_2']
  },
  [AnimationState.HOVER]: {
    duration: 400,
    loop: false,
    animate: true,
    frames: ['hover_1', 'hover_2'],
    returnTo: AnimationState.IDLE
  }
})

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
    this.animationConfig = getAnimationConfig()
  }

  // 切换动画状态
  setState(newState, customFrames = [], force = false) {
    if (this.currentState === newState && !force) return
    
    const oldState = this.currentState
    this.currentState = newState
    this.currentFrame = 0
    this.isAnimating = true
    
    // 如果有自定义帧，更新配置
    if (customFrames && customFrames.length > 0) {
      this.animationConfig[newState].frames = customFrames
    }
    
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
    const config = this.animationConfig[this.currentState]
    if (!config || config.frames.length === 0) {
      console.warn(`没有找到 ${this.currentState} 的动画配置或帧为空`)
      return
    }

    const frameCount = config.frames.length
    const frameDuration = config.duration / frameCount
    
    const animate = () => {
      // 触发帧改变回调
      if (this.callbacks.onFrameChange) {
        this.callbacks.onFrameChange(this.currentState, this.currentFrame)
      }
      
      if (this.animationConfig[this.currentState].animate === false) {
        // 如果不需要动画，直接结束
        this.isAnimating = false
        return
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
    const config = this.animationConfig[this.currentState]
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

// 鼠标事件映射
export const MOUSE_ANIMATION_MAP = {
  'left': AnimationState.LEFT_CLICK,
  'right': AnimationState.RIGHT_CLICK,
  'middle': AnimationState.KEY_PRESS
}

export function getAnimationForMouse(button) {
  return MOUSE_ANIMATION_MAP[button] || AnimationState.KEY_PRESS
}