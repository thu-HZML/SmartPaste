<template>
  <div class="app">
    <!-- 顶部搜索栏 -->
    <header class="app-header">
      <div class="search-container">
        <div class="search-bar">
          <svg class="search-icon" width="20" height="20" viewBox="0 0 100 100">
              <circle cx="40" cy="40" r="30" fill="none" stroke="#3498db" stroke-width="6"/>
                          <line x1="65" y1="65" x2="85" y2="85" stroke="#3498db" stroke-width="6" stroke-linecap="round"/>
          </svg>
          <input 
            type="text" 
            v-model="searchQuery"
            placeholder="搜索剪贴板内容..." 
            class="search-input"
          >
        </div>
      </div>
      
      <div class="toolbar">
        <div class="category-buttons">
          <button 
            v-for="category in categories" 
            :key="category.id"
            :class="['category-btn', { active: activeCategory === category.id }]"
            @click="setActiveCategory(category.id)"
          >
            {{ category.name }}
          </button>
        </div>
        
        <div class="toolbar-actions">
          <button class="icon-btn" @click="togglePinnedView">
            📌
          </button>
          <button class="icon-btn" @click="openSettings">         
            <img
              class="settings-icon"
              src="https://ide.code.fun/api/image?token=69034a079520a30011f4f4f9&name=f8435267bedb1f8da2ed89ce0b7f6027.png"
            />
          </button>
        </div>
      </div>
    </header>

    <!-- 剪贴板记录列表 -->
    <main class="app-main">
      <!-- “全部”界面 -->
      <div v-if="activeCategory === 'all'">
        <div v-if="history.length === 0" class="empty-state">
          <p>暂无剪贴板记录</p>
          <p class="hint">复制的内容将显示在这里</p>
        </div>
        
        <div v-else class="history-list">
          <div 
            v-for="(item, index) in history" 
            :key="index" 
            class="history-item"
            tabindex="0"
          >
            <div class="item-info">
              <div class="item-meta">
                <span>{{ item.tag }}</span>
                <span>{{ item.text.length }}字符</span>
                <span>{{ formatTime(item.timestamp) }}</span>
              </div>

              <!-- 右上方按钮组 -->
              <div class="item-actions-top">
                <button 
                  class="icon-btn-small" 
                  @click="toggleFavorite(index)"
                  :title="item.favorite ? '取消收藏' : '收藏'"
                >
                  {{ item.favorite ? '★' : '☆' }}
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="copyItem(item.text)"
                  title="复制"
                >
                  📋
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="editItem(index)"
                  :disabled="item.text.length > 500"
                >
                  ✏️
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="shareItem(item.text)"
                >
                  📤
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="removeItem(index)"
                >
                  🗑️
                </button>
              </div>
            </div>
            <div class="item-content">
              <div class="item-text" :title="item.text">{{ item.text }}</div>           
            </div>
          </div>
        </div>
      </div>

      <!-- “收藏”界面 -->
      <div v-if="activeCategory === 'favorite'">
        <div class="history-list">
          <div 
            v-for="(item, index) in favoriteHistory" 
            :key="index" 
            class="history-item"
            tabindex="0"
          >
            <div class="item-info">
              <div class="item-meta">
                <span>{{ item.name }}</span>
                <span>{{ item.num }}个内容</span>
              </div>

              <!-- 右上方按钮组 -->
              <div class="item-actions-top">
                <button 
                  class="icon-btn-small" 
                  @click="removeItem(index)"
                >
                  🗑️
                </button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </main>

    <!-- 操作提示 -->
    <div v-if="showToast" class="toast">
      {{ toastMessage }}
    </div>

    <!-- 编辑模态框 -->
    <div v-if="showEditModal" class="modal">
      <div class="modal-content">
        <h3>编辑内容</h3>
        <textarea 
          v-model="editingText" 
          class="edit-textarea"
          placeholder="请输入内容..."
        ></textarea>
        <div class="modal-actions">
          <button @click="cancelEdit" class="btn btn-secondary">取消</button>
          <button @click="saveEdit" class="btn btn-primary">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script>
import { ref, computed, onMounted} from 'vue'
import { useRouter } from 'vue-router'
import { invoke } from '@tauri-apps/api/core'

export default {
  name: 'App',
  setup() {
    const router = useRouter()

    const searchQuery = ref('')
    const activeCategory = ref('all')
    const showToast = ref(false)
    const toastMessage = ref('')
    const showEditModal = ref(false)
    const editingIndex = ref(-1)
    const editingText = ref('')
    
    // 分类选项
    const categories = ref([
      { id: 'all', name: '全部' },
      { id: 'image', name: '图片' },
      { id: 'video', name: '视频' },
      { id: 'favorite', name: '收藏' }
    ])
    
    // 历史记录数据结构
    const history = ref([])
    const favoriteHistory = ref([])

    // 显示提示信息
    const showMessage = (message) => {
      toastMessage.value = message
      showToast.value = true
      setTimeout(() => {
        showToast.value = false
      }, 2000)
    }

    // 设置激活分类
    const setActiveCategory = (categoryId) => {
      activeCategory.value = categoryId
    }

    // 切换固定视图
    const togglePinnedView = () => {
      showMessage('切换固定视图')
    }

    // 打开设置
    const openSettings = () => {
      router.push('/preferences')
      showMessage('打开设置')
    }

    // 过滤后的历史记录
    const filteredHistory = computed(() => {
      let filtered = history.value
      
      // 搜索过滤
      if (searchQuery.value) {
        filtered = filtered.filter(item => 
          item.text.toLowerCase().includes(searchQuery.value.toLowerCase())
        )
      }
      
      // 分类过滤
      switch (activeCategory.value) {
        case 'favorite':
          filtered = filtered.filter(item => item.favorite)
          break
        case 'image':
          // 模拟图片类型过滤
          filtered = filtered.filter(item => item.text.includes('image') || item.text.includes('图片'))
          break
        case 'video':
          // 模拟视频类型过滤
          filtered = filtered.filter(item => item.text.includes('video') || item.text.includes('视频'))
          break
        // 'all' 不进行过滤
      }
      
      return filtered
    })

    // 从剪贴板读取
    const readFromClipboard = async () => {
      try {
        let text = ''
        if (navigator.clipboard && navigator.clipboard.readText) {
          text = await navigator.clipboard.readText()
        } else {
          // 模拟读取
          text = '模拟剪贴板内容 - ' + new Date().toLocaleTimeString()
        }
        
        // 添加到历史记录
        addToHistory(text)
        showMessage('已从剪贴板读取并保存')
      } catch (error) {
        console.error('读取剪贴板失败:', error)
        showMessage('读取剪贴板失败')
      }
    }

    // 添加到历史记录
    const addToHistory = (text) => {
      if (!text.trim()) return
      
      const newItem = {
        text: text.trim(),
        timestamp: new Date().getTime(),
        pinned: false,
        favorite: false
      }
      
      history.value.unshift(newItem)
      // 限制历史记录数量
      if (history.value.length > 100) {
        history.value.pop()
      }
      saveToStorage()
    }

    // 复制项目
    const copyItem = async (text) => {
      try {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          await navigator.clipboard.writeText(text)
        } else {
          // 备用方案
          const textArea = document.createElement('textarea')
          textArea.value = text
          document.body.appendChild(textArea)
          textArea.select()
          document.execCommand('copy')
          document.body.removeChild(textArea)
        }
        showMessage('已复制到剪贴板')
      } catch (error) {
        console.error('复制失败:', error)
        showMessage('复制失败')
      }
    }

    // 切换收藏状态
    const toggleFavorite = (index) => {
      history.value[index].favorite = !history.value[index].favorite
      saveToStorage()
      showMessage(history.value[index].favorite ? '已收藏' : '已取消收藏')
    }

    // 编辑项目
    const editItem = (index) => {
      editingIndex.value = index
      editingText.value = history.value[index].text
      showEditModal.value = true
    }

    // 保存编辑
    const saveEdit = () => {
      if (editingIndex.value >= 0 && editingText.value.trim()) {
        history.value[editingIndex.value].text = editingText.value.trim()
        history.value[editingIndex.value].timestamp = new Date().getTime()
        saveToStorage()
        showMessage('内容已更新')
      }
      cancelEdit()
    }

    // 取消编辑
    const cancelEdit = () => {
      showEditModal.value = false
      editingIndex.value = -1
      editingText.value = ''
    }

    // 分享项目
    const shareItem = (text) => {
      // 模拟分享功能
      if (navigator.share) {
        navigator.share({
          title: '剪贴板内容',
          text: text
        })
      } else {
        showMessage('分享功能不可用')
      }
    }

    // 删除项目
    const removeItem = (index) => {
      history.value.splice(index, 1)
      saveToStorage()
      showMessage('已删除记录')
    }

    // 截断长文本
    const truncateText = (text) => {
      return text
    }

    // 格式化时间
    const formatTime = (timestamp) => {
      const date = new Date(timestamp)
      const now = new Date()
      const diff = now - date
      
      if (diff < 60000) return '刚刚'
      if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`
      if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`
      
      return date.toLocaleDateString()
    }

    // 保存到本地存储
    const saveToStorage = () => {
      localStorage.setItem('clipboardHistory', JSON.stringify(history.value))
    }

    // 从本地存储加载
    const loadFromStorage = () => {
      const saved = localStorage.getItem('clipboardHistory')
      if (saved) {
        history.value = JSON.parse(saved)
      }
    }

    onMounted(() => {
      console.log('开始初始化...')

      // 直接设置数据
      history.value = [
        {
          tag: '纯文本',
          text: '欢迎使用 SmartPaste 剪贴板管理器！',
          timestamp: Date.now(),
          pinned: true,
          favorite: true
        },
        {
          tag: '纯文本',
          text: '测试数据1',
          timestamp: Date.now() - 100000,
          pinned: false,
          favorite: false
        },
        {
          tag: '纯文本',
          text: '测试数据2:长文本测试，这是一条非常长的文本，用于测试文本截断功能。'.repeat(10),
          timestamp: Date.now() - 100000,
          pinned: false,
          favorite: false
        },
        {
          tag: '纯文本',
          text: '测试数据3:中文本测试，这是一条比较长的文本，用于测试文本截断功能。'.repeat(3),
          timestamp: Date.now() - 100000,
          pinned: false,
          favorite: false
        }
      ]
      
      console.log('数据设置完成:', history.value)
      console.log('数据长度:', history.value.length)

      try {
        // 调用 Rust 函数
        history.value = invoke('get_all_data')
      } catch (err) {
        console.error('调用失败:', err)
      }
    })

    return {
      searchQuery,
      activeCategory,
      categories,
      history,
      showToast,
      toastMessage,
      showEditModal,
      editingText,
      setActiveCategory,
      togglePinnedView,
      openSettings,
      copyItem,
      toggleFavorite,
      editItem,
      saveEdit,
      cancelEdit,
      shareItem,
      removeItem,
      truncateText,
      formatTime
    }
  }
}
</script>

<style scoped>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  background: #f8f9fa;
  color: #333;
  line-height: 1.6;
  overflow-x: hidden;
  max-width: 100%;
}

.app {
  min-height: 100vh;
  background: white;
  overflow-x: hidden;
  max-width: 100%;
}

/* 顶部搜索栏样式 */
.app-header {
  background: white;
  border-bottom: 1px solid #e1e8ed;
  padding: 0;
  max-width: 100%
}

.search-container {
  padding: 8px 10px;
  border-bottom: 1px solid #f0f0f0;
}

.search-bar {
  position: relative;
  margin: 0 auto;
}

.search-icon {
  position: absolute;
  left: 16px;
  top: 50%;
  transform: translateY(-50%);
}

/* 搜索框样式 */
.search-input {
  width: 100%;
  padding: 6px 10px 6px 40px;
  border: 1px solid #e1e8ed;
  border-radius: 8px;
  font-size: 16px;
  outline: none;
  transition: all 0.2s;
}

.search-input:hover {
  border-color: #b7c8fe;
}

.search-input:focus {
  border-color: #3282f6;
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.1);
}

/* 工具栏样式 */
.toolbar {
  display: flex;
  justify-content: space-between;
  padding: 8px 10px;
  background: #ffffff;
}

.category-buttons {
  display: flex;
  gap: 8px;
}

.category-btn {
  padding: 4px 8px;
  border: none;
  border-radius: 8px;
  background: white;
  color: #666;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
}

.category-btn:hover {
  background: #f1f3f5;
}

.category-btn.active {
  background: #e4edfd;
  color: #416afe;
}

.toolbar-actions {
  display: flex;
  gap: 8px;
}

.icon-btn {
  padding: 4px;
  border: none;
  background: none;
  font-size: 18px;
  cursor: pointer;
  border-radius: 6px;
  transition: background 0.2s;
}

.icon-btn:hover {
  background: #e9ecef;
}

.settings-icon {
  width: 1.2rem;
  height: 1.2rem;
  position: relative;
  top: 3px;
}

/* 主内容区样式 */
.app-main {
  padding: 8px 10px;
  margin: 0 auto;
}

/* 空状态样式 */
.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #7f8c8d;
}

.empty-state p {
  margin-bottom: 8px;
}

.hint {
  font-size: 14px;
  color: #bdc3c7;
}

/* 历史记录列表样式 */
.history-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.history-item {
  background: white;
  border: 1px solid #e1e8ed;
  border-radius: 12px;
  padding: 2px 5px;
  transition: all 0.2s ease;
  position: relative;
}

.history-item:hover {
  border-color: #b7c8fe;
}

.history-item:focus {
  border-color: #3282f6;
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.1);
}

/* 信息框架 */
.item-info {
  display: flex;
  justify-content: space-between;
}

/* 元信息样式 */
.item-meta {
  display: flex;
  gap: 8px;
  font-size: 11px;
  color: #595959;
  align-items: center;
}

/* 功能样式 */
.item-actions-top {
  display: flex;
  gap: 4px;
}

.icon-btn-small {
  padding: 1px;
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

/* 剪贴内容样式 */
.item-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 12px;
}

.item-text {
  display: -webkit-box;
  line-clamp: 4;          /* 限制显示行数 */
  -webkit-line-clamp: 4;      /* 限制显示行数 */
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  font-size: 14px;
  line-height: 1.5;
  word-break: break-word;
  color: #1f1f1f;
  min-height: 81px;
  max-height: 81px;
}

/* 提示框样式 */
.toast {
  position: fixed;
  bottom: 24px;
  left: 50%;
  transform: translateX(-50%);
  background: rgba(0, 0, 0, 0.8);
  color: white;
  padding: 12px 24px;
  border-radius: 8px;
  font-size: 14px;
  z-index: 1000;
  animation: slideUp 0.3s ease;
}

/* 美化纵向滚动条 */
::-webkit-scrollbar {
  width: 8px;
  height: 8px;
}

::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 10px;
}

::-webkit-scrollbar-thumb {
  background: #c1c1c1;
  border-radius: 10px;
  transition: background 0.3s;
}

::-webkit-scrollbar-thumb:hover {
  background: #a8a8a8;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateX(-50%) translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateX(-50%) translateY(0);
  }
}

/* 模态框样式 */
.modal {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1001;
}

.modal-content {
  background: white;
  border-radius: 12px;
  padding: 24px;
  width: 90%;
  max-width: 500px;
  max-height: 80vh;
  overflow: auto;
}

.modal-content h3 {
  margin-bottom: 16px;
  color: #2c3e50;
}

.edit-textarea {
  width: 100%;
  height: 200px;
  padding: 12px;
  border: 2px solid #e1e8ed;
  border-radius: 8px;
  resize: vertical;
  font-family: inherit;
  font-size: 14px;
  margin-bottom: 20px;
}

.edit-textarea:focus {
  outline: none;
  border-color: #3498db;
}

.modal-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.btn {
  padding: 10px 16px;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.2s ease;
}

.btn-primary {
  background: #3498db;
  color: white;
}

.btn-secondary {
  background: #95a5a6;
  color: white;
}

.btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0,0,0,0.15);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .toolbar {
    gap: 12px;
    align-items: stretch;
  }
  
  .category-buttons {
    justify-content: center;
    flex-wrap: wrap;
  }
  
  .toolbar-actions {
    justify-content: center;
  }
  
  .item-content {
    flex-direction: column;
  }
  
  .item-actions-top {
    align-self: flex-end;
  }
  
  .item-actions-bottom {
    justify-content: flex-start;
    flex-wrap: wrap;
  }
  
  .app-main {
    padding: 16px;
  }
  
  .search-container {
    padding: 12px 16px;
  }
  
  .toolbar {
    padding: 12px 16px;
  }
}
</style>