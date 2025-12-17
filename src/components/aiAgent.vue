<template>
  <!-- AI助手界面 -->
  <div 
    class="ai-assistant"
    ref="aiAssistantRef"
    :class="{ 
      'transparent': isTransparent
    }"
    @mouseenter="handleMouseEnter"
  >
    <!-- 对话框区域 -->
    <div 
      v-if = "hasResponse || loading"
      class="ai-response"
      :style="{ maxHeight: maxResponseHeight + 'px' }"
    >
      <div class="response-content">
        <div v-if="loading" class="loading-indicator">
          <div class="loading-dots">
            <span></span>
            <span></span>
            <span></span>
          </div>
          <span class="loading-text">AI正在思考...</span>
        </div>
        <div v-else class="response-text">
          {{ responseText }}
        </div>
      </div>
      <!-- 复制按钮 -->
      <div v-if="hasResponse && !loading" class="response-actions">
        <button class="icon-btn-small" @click="copyResponse" title="复制回答">
          <Square2StackIcon class="icon-default" />
        </button>
        <button class="icon-btn-small" @click="clearResponse" title="清空">
          <XMarkIcon class="icon-default" />
        </button>
      </div>
    </div>

    <!-- 操作按钮区域 -->
    <div class="ai-actions">
      <button 
        v-for="action in aiActions" 
        :key="action.id"
        :class="['category-btn', { active: currentAction === action.id }]"
        :disabled="loading || !clipboardContent"
        @click="executeAI(action.id)"
      >
        <span class="action-text">{{ action.label }}</span>
        <span v-if="loading && currentAction === action.id" class="action-loading">
          <svg class="spinner" viewBox="0 0 50 50">
            <circle cx="25" cy="25" r="20" fill="none" stroke-width="5"></circle>
          </svg>
        </span>
      </button>
    </div>

    <!-- 关闭按钮 -->
    <button v-if = "!hasResponse && !loading" class="ai-close-btn" @click="closeAI">
      <XMarkIcon class="icon-default" />
    </button>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { Square2StackIcon, XMarkIcon } from '@heroicons/vue/24/outline'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { updateAiWindowHeight } from '../utils/actions.js'
import { apiService } from '../services/api'

// Props
const props = defineProps({
  // 是否显示AI界面
  showAI: {
    type: Boolean,
    default: false
  },
  // 剪贴板内容
  clipboardContent: {
    type: String,
    default: ''
  },
  // 剪贴板内容类型
  clipboardType: {
    type: String,
    default: 'text'
  }
})
const clipboardContent = ref(props.clipboardContent)
const clipboardType = ref(props.clipboardType)
const windowHeight = ref(70)
let resizeObserver = null

// 当前窗口
const currentWindow = getCurrentWindow();

// 状态
const isTransparent = ref(true) // 是否半透明
const hasResponse = ref(false) // 是否有AI回复
const responseText = ref('') // AI回复文本
const loading = ref(false) // 是否正在加载
const currentAction = ref('') // 当前执行的动作
const autoCloseTimer = ref(null) // 自动关闭计时器
const mouseInTimer = ref(null) // 鼠标进入计时器
const isMouseInside = ref(false) // 鼠标是否在区域内
const aiAssistantRef = ref(null)

// 配置
const maxResponseHeight = 400 // 最大响应高度
const autoCloseDelay = 5000 // 自动关闭延迟(5秒)
const fadeInDelay = 300 // 淡入延迟
const maxContentLength = 5000 // 最大处理内容长度

// AI操作列表
const aiActions = ref([
  { id: 'question', label: '提问' },
  { id: 'summarize', label: '总结' },
  { id: 'translate', label: '翻译' },
  { id: 'search', label: '搜索' }
])
/*
// 监听内容变化，更新窗口高度
watch([responseText, hasResponse], async () => {
  console.log('内容发生变化')
  updateWindowHeight()
})
*/
// 更新窗口高度
const updateWindowHeight = async () => {  
  // 获取组件的实际高度
  const newHeight = aiAssistantRef.value.offsetHeight + 2
  
  if (newHeight === windowHeight.value) return
  windowHeight.value = newHeight

  console.log('AI组件高度:', {
    实际高度: newHeight + 'px',
  })
  
  // 可以在这里更新窗口大小
  await updateWindowSize(newHeight)
}

// 更新窗口大小
const updateWindowSize = async (height) => {
  try {   
    const scaleFactor = await currentWindow.scaleFactor()
    // 获取当前窗口大小
    const currentSize = await currentWindow.innerSize()
    console.log('当前窗口大小:', currentSize)

    // 获取主窗口位置，调整AI窗口位置
    const allWindows = await WebviewWindow.getAll()
    const mainWindow = allWindows.find(win => win.label === 'main')

    const physicalPosition = await mainWindow.outerPosition()
    const mainWindowPosition = {
    x: Math.round(physicalPosition.x / scaleFactor),
    y: Math.round(physicalPosition.y / scaleFactor)
    }

    const newX = mainWindowPosition.x - currentSize.width / scaleFactor + 140
    const newY = mainWindowPosition.y - height

    await currentWindow.setPosition(new LogicalPosition(newX, newY))
    console.log('设置新位置')
    // 设置新高度
    await currentWindow.setSize({
      type: 'Logical',
      width: currentSize.width / scaleFactor,
      height: height
    })
    console.log('设置新高度')

    const windowHeight = {
      height: height,
    }

    localStorage.setItem('aiWindowHeight', JSON.stringify(windowHeight))
  } catch (error) {
    console.error('更新窗口大小失败:', error)
  }
}

// 开始自动关闭计时器
const startAutoCloseTimer = () => {
  clearTimers()
  if (!isMouseInside.value) {
    autoCloseTimer.value = setTimeout(() => {
      if (isTransparent.value && !hasResponse.value) {
        closeAI()
      }
    }, autoCloseDelay)
  }
}

// 清空所有计时器
const clearTimers = () => {
  if (autoCloseTimer.value) {
    clearTimeout(autoCloseTimer.value)
    autoCloseTimer.value = null
  }
  if (mouseInTimer.value) {
    clearTimeout(mouseInTimer.value)
    mouseInTimer.value = null
  }
}

// 鼠标进入处理
const handleMouseEnter = () => {
  isMouseInside.value = true
  clearTimers()
  
  if (isTransparent.value) {
    mouseInTimer.value = setTimeout(() => {
      isTransparent.value = false
    }, fadeInDelay)
  }
}

// 执行AI操作
const executeAI = async (action) => {
  if (loading.value) return
  console.log('执行AI操作:', action)
  loading.value = true
  currentAction.value = action
  hasResponse.value = true
  isTransparent.value = false
  
  try {
    var formdata = new FormData();
    let type = 'text'
    let content = clipboardContent.value
    let userQuest = ''
    
    // 根据action类型设置不同的参数
    switch(action) {
      case 'question':
        userQuest = '请回答以下问题：'
        break
        
      case 'summarize':
        userQuest = '请总结以下内容：'
        break
        
      case 'translate':
        userQuest = '请将以下内容翻译成中文：'
        break
        
      case 'search':
        userQuest = `请给出以下与内容相关的网址：`
        break       
    }
    
    formdata.append("type", type)
    formdata.append("content", content)
    formdata.append("user_quest", userQuest)
    formdata.append("provider", "default")

    const response = await apiService.aiChat(formdata)
/*
    const response = await apiService.aiChat({
      type: 'none',
      content: '',
      user_quest: 'hello',
      provider: 'default'
    })*/

    if (response.success) {
      responseText.value = response.data.reply
    }
    
    /*
    // 处理响应
    if (response && response.success) {
      responseText.value = response.result || 'AI未返回有效内容'
      
      // 如果是流式响应，这里可以处理实时更新
      if (response.streaming) {
        // 模拟实时更新（实际使用时需要根据后端API调整）
        simulateStreamingResponse(response.result)
      }
    } else {
      responseText.value = response?.error || 'AI处理失败'
    }*/  
  } catch (error) {
    console.error('AI调用失败:', error)
    responseText.value = `AI服务错误: ${error.message || '未知错误'}`
  } finally {
    loading.value = false
  }
}

// 模拟流式响应（实际使用时根据后端API实现）
const simulateStreamingResponse = (text) => {
  responseText.value = ''
  let index = 0
  const interval = setInterval(() => {
    if (index < text.length) {
      responseText.value += text.charAt(index)
      index++
    } else {
      clearInterval(interval)
    }
  }, 20)
}

// 复制AI回复
const copyResponse = async () => {
  try {
    await navigator.clipboard.writeText(responseText.value)
    showToast('已复制到剪贴板')
  } catch (error) {
    console.error('复制失败:', error)
    showToast('复制失败')
  }
}

// 清空回复
const clearResponse = () => {
  responseText.value = ''
  hasResponse.value = false
  if (!isMouseInside.value) {
    startAutoCloseTimer()
  }
}

// 重置AI状态
const resetAI = () => {
  responseText.value = ''
  hasResponse.value = false
  loading.value = false
  currentAction.value = ''
  isTransparent.value = true
}

// 关闭AI界面
const closeAI = () => {
  resetAI()
  clearTimers()
  currentWindow.close()
}

// 显示提示
const showToast = (message) => {
  // 这里可以集成全局的Toast组件
  console.log('Toast:', message)
}

// 组件挂载和卸载
onMounted( async () => {
  startAutoCloseTimer()

  // 初始化剪贴板内容
  const jsonString = await invoke('get_latest_data')
  const latestData = JSON.parse(jsonString)
  clipboardContent.value = latestData.content
  clipboardType.value = latestData.item_type

  // 创建 ResizeObserver 监听元素尺寸变化
  resizeObserver = new ResizeObserver( async (entries) => {
    for (let entry of entries) {
      const newHeight = entry.contentRect.height + 2
      if (newHeight === windowHeight.value) continue
      windowHeight.value = newHeight
      console.log('元素尺寸变化，新高度:', newHeight, '被观察的组件:', entry)
      await updateWindowSize(newHeight)
    }
  })
  
  // 开始观察
  if (aiAssistantRef.value) {
    resizeObserver.observe(aiAssistantRef.value)
    console.log('正在观察的组件：', aiAssistantRef.value)
  }
})

onUnmounted(() => {
  clearTimers()
})

// 暴露方法供父组件调用
defineExpose({
  resetAI,
  closeAI
})
</script>

<style scoped>
/* AI助手容器 */
.ai-assistant {
  background: white;
  border: 1px solid white;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
  overflow: hidden;
  transition: all 0.3s ease;
  animation: slideInRight 0.3s ease;
}

/* 半透明状态 */
.ai-assistant.transparent {
  opacity: 0.5;
  backdrop-filter: blur(4px);
  background: rgba(255, 255, 255, 0.5);
  border: none;
}

.ai-assistant.transparent:hover {
  opacity: 1;
}

/* 响应区域 */
.ai-response {
  max-height: 400px;
  overflow-y: auto;
  border-bottom: 1px solid #e1e8ed;
  background: #f8f9fa;
  transition: max-height 0.3s ease;
}

.response-content {
  padding: 16px;
  min-height: 60px;
}

/* 加载指示器 */
.loading-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 10px;
  color: #666;
  padding: 20px;
}

.loading-dots {
  display: flex;
  gap: 4px;
}

.loading-dots span {
  width: 8px;
  height: 8px;
  background: #3498db;
  border-radius: 50%;
  animation: bounce 1.4s infinite ease-in-out both;
}

.loading-dots span:nth-child(1) {
  animation-delay: -0.32s;
}

.loading-dots span:nth-child(2) {
  animation-delay: -0.16s;
}

@keyframes bounce {
  0%, 80%, 100% {
    transform: scale(0);
  }
  40% {
    transform: scale(1);
  }
}

.loading-text {
  font-size: 14px;
}

/* 响应文本 */
.response-text {
  font-size: 14px;
  line-height: 1.6;
  color: #333;
  white-space: pre-wrap;
  word-break: break-word;
}

/* 响应操作按钮 */
.response-actions {
  display: flex;
  justify-content: flex-end;
  padding: 8px 16px;
  background: rgba(255, 255, 255, 0.9);
  border-top: 1px solid #e1e8ed;
}

/* AI操作按钮区域 */
.ai-actions {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
}

.category-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 12px 8px;
  border: none;
  border-radius: 8px;
  background: white;
  color: #666;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.category-btn:hover {
  background: #e4edfd;
}

.category-btn.active {
  background: #e4edfd;
  color: #416afe;
}

.category-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.category-btn.loading {
  opacity: 0.7;
}

.action-icon {
  font-size: 20px;
  margin-bottom: 6px;
}

.action-text {
  font-weight: 500;
}

.action-loading {
  position: absolute;
  top: 4px;
  right: 4px;
}

.spinner {
  width: 16px;
  height: 16px;
  animation: rotate 1s linear infinite;
}

.spinner circle {
  stroke: #3498db;
  stroke-linecap: round;
  animation: dash 1.5s ease-in-out infinite;
}

@keyframes rotate {
  100% {
    transform: rotate(360deg);
  }
}

@keyframes dash {
  0% {
    stroke-dasharray: 1, 150;
    stroke-dashoffset: 0;
  }
  50% {
    stroke-dasharray: 90, 150;
    stroke-dashoffset: -35;
  }
  100% {
    stroke-dasharray: 90, 150;
    stroke-dashoffset: -124;
  }
}

/* 关闭按钮 */
.ai-close-btn {
  position: absolute;
  top: 8px;
  right: 8px;
  width: 24px;
  height: 24px;
  border: none;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  z-index: 1003;
  transition: all 0.2s ease;
}

.ai-close-btn:hover {
  background: #f1f3f5;
  transform: scale(1.1);
}

/* 小图标按钮 */
.icon-btn-small {
  padding: 4px;
  border: none;
  background: none;
  font-size: 14px;
  cursor: pointer;
  border-radius: 4px;
  transition: background 0.2s;
}

.icon-btn-small:hover {
  background: #e9ecef;
}

.icon-default {
  width: 1rem;
  height: 1rem;
  color: #595959;
}

.icon-default:hover {
  color: #3282f6;
}

/* 动画 */
@keyframes slideInRight {
  from {
    opacity: 0;
    transform: translateX(20px);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

/* 滚动条美化 */
.ai-response::-webkit-scrollbar {
  width: 6px;
}

.ai-response::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 3px;
}

.ai-response::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 3px;
}

.ai-response::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}
</style>