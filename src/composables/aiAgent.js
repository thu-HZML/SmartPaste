import { ref, computed, watch, onMounted, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow, LogicalPosition } from '@tauri-apps/api/window'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { marked } from 'marked'
import DOMPurify from 'dompurify'
import { apiService } from '../services/api'

export function useAiAgent(props) {
  // 当前窗口
  const currentWindow = getCurrentWindow();
  
  // Refs (对应模板中的 DOM 引用)
  const aiAssistantRef = ref(null)
  const questionInputRef = ref(null)

  // 状态管理
  const clipboardContent = ref(props.clipboardContent || '')
  const clipboardType = ref(props.clipboardType || 'text')
  const windowHeight = ref(70)
  let resizeObserver = null

  const isTransparent = ref(true) // 是否半透明
  const hasResponse = ref(false) // 是否有AI回复
  const responseText = ref('') // AI回复文本
  const loading = ref(false) // 是否正在加载
  const currentAction = ref('') // 当前执行的动作
  const autoCloseTimer = ref(null) // 自动关闭计时器
  const mouseInTimer = ref(null) // 鼠标进入计时器
  const isMouseInside = ref(false) // 鼠标是否在区域内

  // 提问相关状态
  const showQuestionInput = ref(false) // 是否显示提问输入框
  const questionInput = ref('') // 用户输入的问题
  const conversationHistory = ref([]) // 对话历史

  // 流式响应相关状态
  const isStreaming = ref(false) // 是否正在流式输出
  const streamingBuffer = ref('') // 流式缓冲区

  // 配置
  const maxResponseHeight = 700 // 最大响应高度
  const autoCloseDelay = 5000 // 自动关闭延迟(5秒)
  const fadeInDelay = 300 // 淡入延迟

  // AI操作列表
  const aiActions = ref([
    { id: 'question', label: '提问' },
    { id: 'summarize', label: '总结' },
    { id: 'translate', label: '翻译' },
    { id: 'search', label: '搜索' }
  ])

  // 辅助函数：HTML转义
  const escapeHtml = (text) => {
    const div = document.createElement('div')
    div.textContent = text
    return div.innerHTML
  }

  // 计算属性：将Markdown转换为安全的HTML
  const formattedResponse = computed(() => {
    if (!responseText.value) return ''
    
    try {
      marked.setOptions({
        breaks: true,
        gfm: true,
        highlight: (code, lang) => {
          return `<pre><code class="language-${lang}">${escapeHtml(code)}</code></pre>`
        }
      })
      
      const rawHtml = marked(responseText.value)
      
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
      return escapeHtml(responseText.value)
    }
  })

  // 更新窗口大小
  const updateWindowSize = async (height) => {
    try {   
      const scaleFactor = await currentWindow.scaleFactor()
      const currentSize = await currentWindow.innerSize()
      // console.log('当前窗口大小:', currentSize)

      const allWindows = await WebviewWindow.getAll()
      const mainWindow = allWindows.find(win => win.label === 'main')

      // 防止主窗口找不到的情况
      if (!mainWindow) return

      const physicalPosition = await mainWindow.outerPosition()
      const mainWindowPosition = {
        x: Math.round(physicalPosition.x / scaleFactor),
        y: Math.round(physicalPosition.y / scaleFactor)
      }

      const newX = mainWindowPosition.x - currentSize.width / scaleFactor + 140
      const newY = mainWindowPosition.y - height

      await new Promise(resolve => {
        requestAnimationFrame(async () => {
          try {
            await Promise.all([
              currentWindow.setPosition(new LogicalPosition(newX, newY)),
              currentWindow.setSize({
                type: 'Logical',
                width: currentSize.width / scaleFactor,
                height: height
              })
            ])
            // console.log('窗口位置和大小已更新')
            localStorage.setItem('aiWindowHeight', JSON.stringify({ height }))
          } catch (error) {
            console.error('更新窗口失败:', error)
          } finally {
            resolve()
          }
        })
      })
    } catch (error) {
      console.error('更新窗口大小失败:', error)
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

  // 显示提示 (简单的 console log，可扩展)
  const showToast = (message) => {
    console.log('Toast:', message)
  }

  // 调用ai的api
  const useAi = async () => {
    console.log('执行AI操作:', currentAction.value)
    loading.value = true
    isStreaming.value = true
    hasResponse.value = true
    isTransparent.value = false
    
    responseText.value = ''
    streamingBuffer.value = ''

    try {
      var formdata = new FormData();
      let content = clipboardContent.value
      let userQuest = ''
      
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

        const osPath = await invoke('get_config_item', { key: 'storage_path'})
        let filePath = osPath.replace(/\//g, '\\') + '\\' + clipboardContent.value
        console.log('获取的文件路径：', filePath)
        let base64Content = null;
        try {
            base64Content = await invoke('read_file_base64', { filePath });
        } catch (e) {
            console.error('读取本地文件失败:', e);
            showToast('读取本地文件失败，请确保 Rust 命令已实现');
            return false;
        }
        const fileName = filePath.substring(filePath.lastIndexOf('\\') + 1)
        
        const base64Data = base64Content.split(',').pop();
        const binaryString = atob(base64Data);
        const len = binaryString.length;
        const bytes = new Uint8Array(len);
        for (let i = 0; i < len; i++) {
          bytes[i] = binaryString.charCodeAt(i);
        }
        const fileObject = new File([bytes], fileName);
        formdata.append('file', fileObject, fileObject.name); 
      }
      
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
      formdata.append('response_format', 'markdown')

      const response = await apiService.aiChat(formdata, (chunk, fullText) => {
        loading.value = false
        streamingBuffer.value += chunk
        responseText.value = streamingBuffer.value
        
        nextTick(() => {
          const responseEl = aiAssistantRef.value?.querySelector('.ai-response')
          if (responseEl) {
            responseEl.scrollTop = responseEl.scrollHeight
          }
        })
      })

      if (response.success) {
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

  // 提交用户提问
  const submitQuestion = async () => {
    if (!questionInput.value.trim() || loading.value) return
    
    console.log('提交问题:', questionInput.value)
    showQuestionInput.value = false
    
    try {
      let aiChatSuccess = await useAi()
      if (aiChatSuccess) {
        // 可以在这里处理历史记录
      }
    } catch (error) {
      console.error('AI调用失败:', error)
    } finally {
      loading.value = false
      questionInput.value = ''
    }
  }

  // 执行AI操作
  const executeAI = async (action) => {
    if (loading.value) return

    currentAction.value = action

    if (action === 'question') {
      showQuestionInput.value = true
      hasResponse.value = true
      isTransparent.value = false
      questionInput.value = ''
      
      nextTick(() => {
        if (questionInputRef.value) {
          questionInputRef.value.focus()
          questionInputRef.value.select()
        }
      })
      return
    }

    await useAi()
  }

  // 取消提问
  const cancelQuestion = () => {
    showQuestionInput.value = false
    questionInput.value = ''
    
    if (!responseText.value) {
      hasResponse.value = false
      if (!isMouseInside.value) {
        startAutoCloseTimer()
      }
    }
  }

  // 继续提问
  const askFollowUp = () => {
    showQuestionInput.value = true
    hasResponse.value = true
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

  // 生命周期
  onMounted(async () => {
    startAutoCloseTimer()

    // 初始化剪贴板内容
    try {
      const jsonString = await invoke('get_latest_data')
      const latestData = JSON.parse(jsonString)
      clipboardContent.value = latestData.content
      clipboardType.value = latestData.item_type
      console.log('接收到的信息：', clipboardType.value, clipboardContent.value)
    } catch (e) {
      console.error("获取初始数据失败", e)
    }

    // 监听尺寸变化
    resizeObserver = new ResizeObserver(async (entries) => {
      for (let entry of entries) {
        const newHeight = entry.contentRect.height + 2
        if (newHeight === windowHeight.value) continue
        windowHeight.value = newHeight
        await updateWindowSize(newHeight)
      }
    })
    
    if (aiAssistantRef.value) {
      resizeObserver.observe(aiAssistantRef.value)
    }
  })

  onUnmounted(() => {
    clearTimers()
    if (resizeObserver && aiAssistantRef.value) {
      resizeObserver.unobserve(aiAssistantRef.value)
      resizeObserver.disconnect()
    }
  })

  return {
    // Refs
    aiAssistantRef,
    questionInputRef,
    
    // State
    isTransparent,
    hasResponse,
    loading,
    currentAction,
    showQuestionInput,
    questionInput,
    clipboardContent,
    maxResponseHeight,
    aiActions,
    
    // Computed
    formattedResponse,
    
    // Methods
    handleMouseEnter,
    submitQuestion,
    cancelQuestion,
    copyResponse,
    clearResponse,
    askFollowUp,
    executeAI,
    closeAI,
    resetAI
  }
}