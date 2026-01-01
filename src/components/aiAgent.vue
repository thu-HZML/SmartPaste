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
import { Square2StackIcon, XMarkIcon, PaperAirplaneIcon, ChatBubbleLeftRightIcon } from '@heroicons/vue/24/outline'
import { useAiAgent } from '../composables/aiAgent'

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

// 使用组合式函数
const {
  aiAssistantRef,
  questionInputRef,
  isTransparent,
  hasResponse,
  loading,
  currentAction,
  showQuestionInput,
  questionInput,
  clipboardContent,
  maxResponseHeight,
  aiActions,
  formattedResponse,
  handleMouseEnter,
  submitQuestion,
  cancelQuestion,
  copyResponse,
  clearResponse,
  askFollowUp,
  executeAI,
  closeAI,
  resetAI
} = useAiAgent(props)

// 暴露方法供父组件调用
defineExpose({
  resetAI,
  closeAI
})
</script>

<style scoped>
/* AI助手容器 - 确保不超出 */
.ai-assistant {
  background: white;
  border: 1px solid white;
  border-radius: 12px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.1);
  overflow: hidden; /* 确保内容不会溢出容器 */
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
  overflow-x: hidden; /* 禁止水平滚动 */
  border-bottom: 1px solid #e1e8ed;
  background: #f8f9fa;
  transition: max-height 0.3s ease;
  width: 100%; /* 确保宽度100% */
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

/* Markdown容器 - 确保内容不溢出 */
.response-text.markdown-body {
  font-size: 14px;
  line-height: 1.6;
  color: #333;
  
  /* 关键修复：确保内容宽度不超过容器 */
  max-width: 100%;
  width: 100%;
  box-sizing: border-box;
  
  /* 确保内容换行 */
  overflow-wrap: break-word;
  word-wrap: break-word;
  word-break: break-word;
  
  /* 防止代码块和表格溢出 */
  overflow: hidden;
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

/* 改进Markdown样式，确保所有元素都遵守容器宽度 */

/* 1. 代码块 - 关键修复 */
.markdown-body pre {
  max-width: 100%;
  width: 100%;
  box-sizing: border-box;
  overflow-x: auto; /* 代码块内部可以水平滚动 */
  white-space: pre-wrap; /* 允许代码换行 */
  word-break: break-all; /* 强制长代码换行 */
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
  display: block;
  max-width: 100%;
  white-space: pre-wrap; /* 代码可以换行 */
  word-break: break-all; /* 强制换行 */
  overflow-wrap: break-word;
}

.markdown-body code:not(pre code) {
  /* 行内代码 */
  white-space: normal;
  word-break: break-word;
}

/* 2. 表格 - 关键修复 */
.markdown-body table {
  display: block;
  max-width: 100%;
  width: 100%;
  overflow-x: auto; /* 表格内部可以水平滚动 */
  border-collapse: collapse;
  margin-bottom: 16px;
}

.markdown-body th,
.markdown-body td {
  max-width: 200px; /* 限制单元格最大宽度 */
  min-width: 60px; /* 保持最小宽度 */
  word-break: break-word;
  white-space: normal; /* 允许换行 */
}

/* 3. 图片 - 确保不超出 */
.markdown-body img {
  max-width: 100%;
  height: auto;
  display: block;
}

/* 4. 长链接和URL - 确保换行 */
.markdown-body a {
  word-break: break-all;
  overflow-wrap: anywhere;
}

/* 5. 列表和段落 */
.markdown-body ul,
.markdown-body ol,
.markdown-body p,
.markdown-body blockquote {
  max-width: 100%;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

/* 6. 标题 */
.markdown-body h1,
.markdown-body h2,
.markdown-body h3,
.markdown-body h4,
.markdown-body h5,
.markdown-body h6 {
  max-width: 100%;
  word-wrap: break-word;
  overflow-wrap: break-word;
}

/* 7. 通用修复：确保所有子元素都继承宽度限制 */
.markdown-body > * {
  max-width: 100%;
  box-sizing: border-box;
}

/* 响应内容区域 - 确保宽度100% */
.response-content {
  padding: 16px;
  min-height: 60px;
  width: 100%;
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

/* 修改滚动条样式，只显示垂直滚动条 */
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

/* 隐藏水平滚动条（如果有的话） */
.ai-response::-webkit-scrollbar:horizontal {
  display: none;
  height: 0;
}
</style>