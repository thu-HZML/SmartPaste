<template>
  <div class="live2d-container">
    <div class="controls" v-if="showControls">
      <button @click="changeExpression">换表情</button>
      <button @click="toggleMotion">切换动作</button>
      <button @click="resetModel">重置</button>
    </div>
    <div class="status" v-if="loading">
      加载中...
    </div>
    <div class="error" v-if="error">
      {{ error }}
    </div>
    <div class="info" v-if="info">
      {{ info }}
    </div>
  </div>
</template>

<script setup>
import { onMounted, onUnmounted, ref } from 'vue'
import live2d from '../utils/live2dManager.js'

// 状态
const showControls = ref(true)
const modelInfo = ref(null)
const motions = ref([])
const expressions = ref([])
const loading = ref(false)
const error = ref(null)
const info = ref(null)

// 加载模型
const loadModel = async () => {
  try {
    console.log('开始加载模型...')
    
    // 使用绝对路径 - 之前成功过的路径
    const modelPath = 'C:/Users/heyufei/Desktop/bigHW/SmartPaste/public/resources/live2d'
    console.log('使用路径:', modelPath)
    info.value = '正在加载模型...'
    
    const result = await live2d.load(modelPath)
    
    modelInfo.value = result
    motions.value = result.motions ? Object.keys(result.motions) : []
    expressions.value = result.expressions || []
    
    // 初始调整大小
    setTimeout(() => {
      live2d.resizeModel()
      info.value = '模型加载成功！'
    }, 100)
    
    console.log('模型加载成功', result)
  } catch (err) {
    console.error('加载模型失败:', err)
  }
}

// 切换动作
const toggleMotion = () => {
  if (!motions.value.length) {
    console.warn('没有可用动作')
    info.value = '没有可用动作'
    return
  }
  
  // 获取第一个动作组
  const motionGroup = motions.value[0]
  if (motionGroup && modelInfo.value?.motions?.[motionGroup]) {
    const motionList = modelInfo.value.motions[motionGroup]
    if (motionList && motionList.length > 0) {
      const randomIndex = Math.floor(Math.random() * motionList.length)
      live2d.playMotion(motionGroup, randomIndex)
      info.value = `播放动作: ${motionGroup}[${randomIndex}]`
    }
  }
}

// 切换表情
const changeExpression = () => {
  if (!expressions.value.length) {
    console.warn('没有可用表情')
    info.value = '没有可用表情'
    return
  }
  
  const randomIndex = Math.floor(Math.random() * expressions.value.length)
  live2d.playExpressions(randomIndex)
  info.value = `播放表情: ${randomIndex}`
}

// 重置模型
const resetModel = () => {
  info.value = '重置模型中...'
  loadModel()
}

// 窗口大小变化处理
const handleResize = () => {
  if (modelInfo.value) {
    setTimeout(() => {
      live2d.resizeModel()
    }, 100)
  }
}

// 组件挂载
onMounted(async () => {
  console.log('Live2D组件挂载')
  
  // 监听窗口大小变化
  window.addEventListener('resize', handleResize)
  
  // 延迟加载模型，确保DOM已渲染
  setTimeout(async () => {
    await loadModel()
  }, 1000)
})

// 组件卸载
onUnmounted(() => {
  console.log('Live2D组件卸载')
  window.removeEventListener('resize', handleResize)
  
  // 清理资源
  try {
    live2d.destroy()
  } catch (err) {
    console.warn('清理资源时出错:', err)
  }
})
</script>

<style scoped>
.live2d-container {
  position: relative;
  width: 100%;
  height: 600px;
  overflow: hidden;
  background: #f5f5f5;
  border-radius: 10px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
}

.controls {
  position: absolute;
  bottom: 20px;
  left: 50%;
  transform: translateX(-50%);
  display: flex;
  gap: 10px;
  background: rgba(255, 255, 255, 0.95);
  padding: 15px 25px;
  border-radius: 30px;
  box-shadow: 0 4px 15px rgba(0, 0, 0, 0.15);
  z-index: 10;
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.2);
}

.controls button {
  padding: 10px 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.3s ease;
  font-size: 14px;
  font-weight: 500;
  letter-spacing: 0.5px;
  box-shadow: 0 4px 15px rgba(102, 126, 234, 0.4);
}

.controls button:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(102, 126, 234, 0.6);
}

.controls button:active {
  transform: translateY(0);
}

.status {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(0, 0, 0, 0.7);
  color: white;
  padding: 20px 40px;
  border-radius: 10px;
  font-size: 18px;
  z-index: 5;
}

.error {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  background: rgba(231, 76, 60, 0.9);
  color: white;
  padding: 20px 40px;
  border-radius: 10px;
  font-size: 16px;
  text-align: center;
  max-width: 80%;
  white-space: pre-line;
  z-index: 5;
}

.info {
  position: absolute;
  top: 20px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(52, 152, 219, 0.9);
  color: white;
  padding: 10px 20px;
  border-radius: 5px;
  font-size: 14px;
  text-align: center;
  max-width: 90%;
  z-index: 5;
}

#live2dCanvas {
  display: block;
  width: 100%;
  height: 100%;
}
</style>