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
      v-if = "hasResponse || loading || showQuestionInput"
      class="ai-response"
      :style="{ maxHeight: maxResponseHeight + 'px' }"
    >
      <!-- 提问输入框区域 -->
      <div v-if="showQuestionInput && !loading" class="question-input-area">
        <input
          ref="questionInputRef"
          v-model="questionInput"
          type="text"
          placeholder="请输入您的问题..."
          class="question-input"
          @keyup.enter="submitQuestion"
          @keyup.esc="cancelQuestion"
        />
        <div class="question-actions">
          <button class="icon-btn-small" @click="submitQuestion" title="发送">
            <PaperAirplaneIcon class="icon-default" />
          </button>
          <button class="icon-btn-small" @click="cancelQuestion" title="取消">
            <XMarkIcon class="icon-default" />
          </button>
        </div>
      </div>
      
      <div class="response-content">
        <div v-if="loading" class="loading-indicator">
          <div class="loading-dots">
            <span></span>
            <span></span>
            <span></span>
          </div>
          <span class="loading-text">AI正在思考...</span>
        </div>
        <!-- 修改这里：使用v-html渲染Markdown -->
        <div 
          v-else-if="hasResponse" 
          class="response-text markdown-body"
          v-html="formattedResponse"
        ></div>
      </div>
      
      <!-- 响应操作按钮 -->
      <div v-if="hasResponse && !loading && !showQuestionInput" class="response-actions">
        <button class="icon-btn-small" @click="copyResponse" title="复制回答">
          <Square2StackIcon class="icon-default" />
        </button>
        <button class="icon-btn-small" @click="clearResponse" title="清空">
          <XMarkIcon class="icon-default" />
        </button>
        <button v-if="hasResponse" class="icon-btn-small" @click="askFollowUp" title="继续提问">
          <ChatBubbleLeftRightIcon class="icon-default" />
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
    <button v-if = "!hasResponse && !loading && !showQuestionInput" class="ai-close-btn" @click="closeAI">
      <XMarkIcon class="icon-default" />
    </button>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { Square2StackIcon, XMarkIcon, PaperAirplaneIcon, ChatBubbleLeftRightIcon } from '@heroicons/vue/24/outline'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { marked } from 'marked' // 添加marked库用于Markdown解析
import DOMPurify from 'dompurify' // 添加DOMPurify用于HTML净化
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

// 提问相关状态
const showQuestionInput = ref(false) // 是否显示提问输入框
const questionInput = ref('') // 用户输入的问题
const questionInputRef = ref(null) // 输入框引用
const conversationHistory = ref([]) // 对话历史

// 流式响应相关状态
const isStreaming = ref(false) // 是否正在流式输出
const streamingBuffer = ref('') // 流式缓冲区

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

// 计算属性：将Markdown转换为安全的HTML
const formattedResponse = computed(() => {
  if (!responseText.value) return ''
  
  try {
    // 配置marked选项
    marked.setOptions({
      breaks: true, // 启用换行符转换
      gfm: true, // 启用GitHub风格的Markdown
      highlight: (code, lang) => {
        // 这里可以添加代码高亮功能
        return `<pre><code class="language-${lang}">${escapeHtml(code)}</code></pre>`
      }
    })
    
    // 将Markdown转换为HTML
    const rawHtml = marked(responseText.value)
    
    // 净化HTML，防止XSS攻击
    return DOMPurify.sanitize(rawHtml, {
      ALLOWED_TAGS: [
        'h1', 'h2', 'h3', 'h4', 'h5', 'h6',
        'p', 'br', 'hr',
        'strong', 'b', 'em', 'i', 'u', 's', 'del',
        'code', 'pre', 'blockquote',
        'ul', 'ol', 'li',
        'table', 'thead', 'tbody', 'tr', 'th', 'td',
        'a', 'img',
        'div', 'span'
      ],
      ALLOWED_ATTR: ['href', 'target', 'rel', 'src', 'alt', 'title', 'class', 'id']
    })
  } catch (error) {
    console.error('Markdown解析失败:', error)
    // 如果解析失败，返回原始文本
    return escapeHtml(responseText.value)
  }
})

// 辅助函数：HTML转义
const escapeHtml = (text) => {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

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

    // 使用 requestAnimationFrame 确保在同一帧中执行位置和大小更新
    await new Promise(resolve => {
      requestAnimationFrame(async () => {
        try {
          // 同时更新位置和大小
          await Promise.all([
            currentWindow.setPosition(new LogicalPosition(newX, newY)),
            currentWindow.setSize({
              type: 'Logical',
              width: currentSize.width / scaleFactor,
              height: height
            })
          ])
          
          console.log('窗口位置和大小已更新')
          
          // 存储窗口高度
          const windowHeight = {
            height: height,
          }
          localStorage.setItem('aiWindowHeight', JSON.stringify(windowHeight))
          
        } catch (error) {
          console.error('更新窗口失败:', error)
        } finally {
          resolve()
        }
      })
    })/*
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
    */
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

  currentAction.value = action

  // 对于提问按钮，显示输入框而不是直接调用API
  if (action === 'question') {
    showQuestionInput.value = true
    hasResponse.value = true
    isTransparent.value = false
    
    // 设置输入框默认值（可选）
    questionInput.value = ''
    
    // 延迟聚焦输入框
    nextTick(() => {
      if (questionInputRef.value) {
        questionInputRef.value.focus()
        // 选中所有文本
        questionInputRef.value.select()
      }
    })
    
    return
  }

  await useAi()
}

// 提交用户提问
const submitQuestion = async () => {
  if (!questionInput.value.trim() || loading.value) return
  
  console.log('提交问题:', questionInput.value)
  showQuestionInput.value = false
  
  try {
    let aiChatSuccess = await useAi()

    if (aiChatSuccess) {
      // 这里可以添加对话历史的处理
    }
    
  } catch (error) {
    console.error('AI调用失败:', error)
  } finally {
    loading.value = false
    questionInput.value = '' // 清空输入框
  }
}

// 调用ai的api
const useAi = async () => {
  console.log('执行AI操作:', currentAction.value)
  loading.value = true
  isStreaming.value = true
  hasResponse.value = true
  isTransparent.value = false
  
  // 清空之前的响应
  responseText.value = ''
  streamingBuffer.value = ''

  try {
    var formdata = new FormData();
    let type = ''
    let content = clipboardContent.value
    let userQuest = ''
    
    // 根据action类型设置不同的参数
    switch(currentAction.value) {
      case 'question':
        userQuest = '请用Markdown格式回答以下问题：' + questionInput.value
        break
        
      case 'summarize':
        userQuest = '请用Markdown格式总结以下内容：'
        break
        
      case 'translate':
        userQuest = '请将以下内容翻译成中文，使用Markdown格式：'
        break
        
      case 'search':
        userQuest = `请给出以下与内容相关的网址，使用Markdown格式：`
        break       
    }
    
    if (clipboardType.value === 'text') {
      formdata.append("type", 'text')
      formdata.append("content", content)
      formdata.append("user_quest", userQuest)
    }
    else if (clipboardType.value === 'file') {
      formdata.append("type", 'file')
      formdata.append("content", '')
      formdata.append("user_quest", userQuest)

      // 读取文件内容为 Base64 编码字符串
      const osPath = await invoke('get_config_item', { key: 'storage_path'})
      let filePath = osPath.replace(/\//g, '\\') + '\\' + clipboardContent.value
      console.log('获取的文件路径：', filePath)
      let base64Content = null;
      try {
          base64Content = await invoke('read_file_base64', { filePath });
      } catch (e) {
          console.error('读取本地文件失败:', e);
          showMessage('读取本地文件失败，请确保 Rust 命令已实现', 'error');
          return;
      }
      const fileName = filePath.substring(filePath.lastIndexOf('\\') + 1)
      
      // 将 Base64 转换为 File 对象
      const base64Data = base64Content.split(',').pop();
      const binaryString = atob(base64Data);
      const len = binaryString.length;
      const bytes = new Uint8Array(len);
      for (let i = 0; i < len; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }
      // 创建 File 对象，供 fetch API 上传
      const fileObject = new File([bytes], fileName);
      console.log('文件：', fileObject)
      formdata.append('file', fileObject, fileObject.name); 
    }
    
    // ai的其他参数
    const aiProvider = await invoke('get_config_item', { key: 'ai_provider'})
    const aiApiKey = await invoke('get_config_item', { key: 'ai_api_key'})
    const aiModel = await invoke('get_config_item', { key: 'ai_model'})
    const aiBaseUrl = await invoke('get_config_item', { key: 'ai_base_url'})
    const aiTemperature = await invoke('get_config_item', { key: 'ai_temperature'})

    let aiConfig = null
    if (aiProvider === 'default') {
      aiConfig = {
        model: aiModel,
        ai_temperature: aiTemperature
      }
    } else {
      aiConfig = {
        api_key: aiApiKey,
        base_url: aiBaseUrl,
        model: aiModel,
        ai_temperature: aiTemperature
      }
    }

    formdata.append('provider', aiProvider)
    formdata.append('ai_config', JSON.stringify(aiConfig))

    // 修改请求，告诉API我们需要Markdown格式的回复
    formdata.append('response_format', 'markdown')

    const response = await apiService.aiChat(formdata, (chunk, fullText) => {
      loading.value = false
      // 流式输出回调
      streamingBuffer.value += chunk
      responseText.value = streamingBuffer.value
      
      // 如果内容很多，自动滚动到底部
      nextTick(() => {
        const responseEl = aiAssistantRef.value?.querySelector('.ai-response')
        if (responseEl) {
          responseEl.scrollTop = responseEl.scrollHeight
        }
      })
    })

    if (response.success) {
      // 如果流式输出过程中没有设置响应文本，使用最终结果
      if (!responseText.value && response.data?.reply) {
        responseText.value = response.data.reply
      }
      return true
    } else {
      responseText.value = response.data || 'AI处理失败'
    }
  } catch (error) {
    console.error('AI调用失败:', error)
    responseText.value = `AI服务错误: ${error.message || '未知错误'}`
  } finally {
    loading.value = false
  }

  return false
}

// 取消提问
const cancelQuestion = () => {
  showQuestionInput.value = false
  questionInput.value = ''
  
  // 如果没有响应文本，可以关闭响应区域
  if (!responseText.value) {
    hasResponse.value = false
    if (!isMouseInside.value) {
      startAutoCloseTimer()
    }
  }
}

// 继续提问（跟进问题）
const askFollowUp = () => {
  showQuestionInput.value = true
  hasResponse.value = true
  
  // 延迟聚焦输入框
  nextTick(() => {
    if (questionInputRef.value) {
      questionInputRef.value.focus()
    }
  })
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
  showQuestionInput.value = false
  questionInput.value = ''
  conversationHistory.value = []
  
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
  showQuestionInput.value = false
  questionInput.value = ''
  conversationHistory.value = []
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
  console.log('接收到的信息：', clipboardType.value, clipboardContent.value)

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

/* 提问输入框区域 */
.question-input-area {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 16px;
  border-bottom: 1px solid #e1e8ed;
  background: white;
}

.question-input {
  flex: 1;
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.question-input:focus {
  border-color: #3498db;
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.2);
}

.question-input::placeholder {
  color: #999;
}

.question-actions {
  display: flex;
  gap: 4px;
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

/* 响应文本 - 修改为Markdown样式，添加溢出处理 */
.response-text.markdown-body {
  font-size: 14px;
  line-height: 1.6;
  color: #333;
  
  /* 确保内容不超出容器 */
  overflow-wrap: break-word; /* 长单词换行 */
  word-wrap: break-word; /* 旧版浏览器兼容 */
  word-break: break-word; /* 更激进的断词策略 */
  max-width: 100%; /* 确保不超出父容器 */
}

/* Markdown样式 - 添加溢出处理 */
.markdown-body h1,
.markdown-body h2,
.markdown-body h3,
.markdown-body h4,
.markdown-body h5,
.markdown-body h6 {
  margin-top: 24px;
  margin-bottom: 16px;
  font-weight: 600;
  line-height: 1.25;
  
  /* 标题溢出处理 */
  overflow-wrap: break-word;
  word-wrap: break-word;
  max-width: 100%;
}

.markdown-body h1 {
  font-size: 1.5em;
  color: #24292e;
  border-bottom: 1px solid #eaecef;
  padding-bottom: 0.3em;
}

.markdown-body h2 {
  font-size: 1.25em;
  color: #24292e;
  border-bottom: 1px solid #eaecef;
  padding-bottom: 0.3em;
}

.markdown-body h3 {
  font-size: 1.1em;
  color: #24292e;
}

.markdown-body h4 {
  font-size: 1em;
  color: #24292e;
}

.markdown-body h5 {
  font-size: 0.875em;
  color: #6a737d;
}

.markdown-body h6 {
  font-size: 0.85em;
  color: #6a737d;
}

.markdown-body p {
  margin-top: 0;
  margin-bottom: 16px;
  color: #333;
  
  /* 段落溢出处理 */
  overflow-wrap: break-word;
  word-wrap: break-word;
  max-width: 100%;
}

.markdown-body blockquote {
  margin: 0;
  padding: 0 1em;
  color: #6a737d;
  border-left: 0.25em solid #dfe2e5;
  background: #f8f9fa;
  border-radius: 4px;
  
  /* 引用块溢出处理 */
  overflow-wrap: break-word;
  word-wrap: break-word;
  max-width: 100%;
}

.markdown-body ul,
.markdown-body ol {
  margin-top: 0;
  margin-bottom: 16px;
  padding-left: 2em;
  color: #333;
  
  /* 列表溢出处理 */
  overflow-wrap: break-word;
  word-wrap: break-word;
  max-width: 100%;
}

.markdown-body li {
  margin-bottom: 0.25em;
  
  /* 列表项溢出处理 */
  overflow-wrap: break-word;
  word-wrap: break-word;
  max-width: 100%;
}

.markdown-body li > p {
  margin-top: 0;
  margin-bottom: 0;
}

/* 代码块溢出处理 - 重点修复 */
.markdown-body code {
  padding: 0.2em 0.4em;
  margin: 0;
  font-size: 85%;
  background-color: rgba(27, 31, 35, 0.05);
  border-radius: 3px;
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, Courier, monospace;
  color: #e83e8c;
  
  /* 行内代码溢出处理 */
  white-space: pre-wrap; /* 保留空格但允许换行 */
  word-break: break-all; /* 强制长代码换行 */
  max-width: 100%;
}

.markdown-body pre {
  padding: 12px;
  overflow: auto;
  font-size: 85%;
  line-height: 1.45;
  background-color: #f6f8fa;
  border-radius: 6px;
  margin-bottom: 16px;
  
  /* 代码块容器 */
  max-width: 100%;
}

.markdown-body pre code {
  padding: 0;
  margin: 0;
  background-color: transparent;
  border: 0;
  font-size: 100%;
  color: #333;
  
  /* 代码块内容 */
  white-space: pre-wrap; /* 保留格式但允许换行 */
  word-break: break-word; /* 强制长内容换行 */
  overflow-wrap: break-word;
  display: block;
  max-width: 100%;
}

.markdown-body a {
  color: #0366d6;
  text-decoration: none;
  
  /* 链接溢出处理 */
  overflow-wrap: break-word;
  word-wrap: break-word;
  max-width: 100%;
  display: inline-block; /* 确保长URL可以正确换行 */
}

.markdown-body a:hover {
  text-decoration: underline;
  color: #0056b3;
}

.markdown-body strong,
.markdown-body b {
  font-weight: 600;
  color: #24292e;
}

.markdown-body em,
.markdown-body i {
  font-style: italic;
  color: #24292e;
}

.markdown-body hr {
  height: 0.25em;
  padding: 0;
  margin: 24px 0;
  background-color: #e1e4e8;
  border: 0;
  border-radius: 2px;
}

/* 表格溢出处理 - 重点修复 */
.markdown-body table {
  display: block;
  width: 100%;
  overflow-x: auto; /* 允许水平滚动 */
  margin-top: 0;
  margin-bottom: 16px;
  border-spacing: 0;
  border-collapse: collapse;
  
  /* 确保表格不会超出容器 */
  max-width: 100%;
}

.markdown-body th {
  font-weight: 600;
  background-color: #f6f8fa;
}

.markdown-body th,
.markdown-body td {
  padding: 6px 13px;
  border: 1px solid #dfe2e5;
  
  /* 表格单元格溢出处理 */
  overflow-wrap: break-word;
  word-wrap: break-word;
  max-width: 200px; /* 限制单元格最大宽度 */
  min-width: 80px; /* 确保单元格有最小宽度 */
}

.markdown-body tr {
  background-color: #fff;
  border-top: 1px solid #c6cbd1;
}

.markdown-body tr:nth-child(2n) {
  background-color: #f6f8fa;
}

/* 对于特别长的单词或URL，添加通用断词规则 */
.markdown-body {
  /* 处理长单词和URL */
  overflow-wrap: anywhere;
  hyphens: auto; /* 自动断字（如果浏览器支持） */
}

/* 确保所有元素都有最大宽度限制 */
.markdown-body > * {
  max-width: 100%;
  box-sizing: border-box;
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