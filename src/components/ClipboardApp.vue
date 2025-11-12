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
            <LockClosedIcon class="icon-settings" />
          </button>
          <button class="icon-btn" @click="openSettings">         
            <Cog6ToothIcon class="icon-settings" />
          </button>
          <button class="icon-btn" @click="refreshPage">           
            <ArrowPathIcon class="icon-settings" />
          </button>
          <button class="icon-btn" @click="deleteAllHistory">           
            <TrashIcon class="icon-settings" />
          </button>
        </div>
      </div>
    </header>

    <!-- 剪贴板记录列表 -->
    <main class="app-main">
      <!-- "全部"、"图片"、"视频"、"文件"界面 -->
      <div v-if="['all', 'image', 'video', 'file'].includes(activeCategory)">
        <div v-if="filteredHistory.length === 0" class="empty-state">
          <p v-if="searchQuery">未找到匹配的记录</p>
          <p v-else>暂无剪贴板记录</p>
          <p class="hint">复制的内容将显示在这里</p>
        </div>
        
        <div v-else class="history-list">
          <div 
            v-for="(item, index) in filteredHistory" 
            :key="index" 
            class="history-item"
            tabindex="0"
            @mouseenter="item.is_focus = true"
            @mouseleave="item.is_focus = false"
          >
            <div class="item-info">
              <div class="item-meta">
                <span>{{ item.item_type }}</span>
                <span>{{ item.content.length }}字符</span>
                <span>{{ formatTime(item.timestamp) }}</span>
              </div>

              <!-- 右上方按钮组 -->
              <div class="item-actions-top">
                <button 
                  class="icon-btn-small" 
                  @click="toggleFavorite(index)"
                  :title="item.is_favorite ? '取消收藏' : '收藏'"
                >
                  <StarIconSolid v-if="item.is_favorite" class="icon-star-solid" />
                  <StarIcon v-else class="icon-default" />
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="copyItem(item)"
                  title="复制"
                >
                  <Square2StackIcon class="icon-default" />
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="editItem(index)"
                  title="编辑"
                  :disabled="item.content.length > 500"
                >
                  <ClipboardIcon class="icon-default" />
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="noteItem(index)"
                  title="备注"
                >
                  <PencilSquareIcon class="icon-default" />
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="removeItem(index)"
                  title="删除"
                >
                  <TrashIcon class="icon-default" />
                </button>
              </div>
            </div>
            <div class="item-content"> 
              <transition name="fade" mode="out-in">               
                  <div v-if="item.is_focus || !item.notes" class="item-text">

                    <!-- 显示文本 -->
                    <div v-if="item.item_type === 'text'" :title="item.content">
                      {{ item.content }}
                    </div>
                    
                    <!-- 显示图片 -->
                    <div v-else-if="item.item_type === 'image'" class="image-container">
                      <img 
                        v-if="item.content"
                        :src="convertFileSrc(item.content)" 
                        :alt="'图片: ' + getFileName(item.content)"
                        class="preview-image"
                        @error="handleImageError"
                      />
                      <div v-else class="loading">加载中...</div>
                      <div class="image-filename">{{ getFileName(item.content) }}</div>
                    </div>

                    <!-- 显示文件 -->
                    <div v-else-if="item.item_type === 'file'" class="file-container">
                      <div class="file-icon">
                        <!-- 可以根据文件类型显示不同的图标 -->
                        <span v-if="isDocumentFile(item.content)" class="icon">📄</span>
                        <span v-else class="icon">📎</span>
                      </div>
                      <div class="file-info">
                        <div class="file-name">{{ getFileName(item.content) }}</div>
                      </div>
                    </div>

                    <!-- 未知类型 -->
                    <div v-else :title="item.content">
                      {{ item.content }}
                    </div>
                  </div>
                  <div v-else class="item-text">
                    {{ item.notes }}
                  </div>
              </transition> 
            </div>    
          </div>
        </div>
      </div>

      <!-- "收藏"界面 -->
      <div v-if="activeCategory === 'favorite'">
        <div v-if="favoriteHistory.length === 0" class="empty-state">
          <p>暂无收藏记录</p>
        </div>
        <div v-else class="history-list">
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
                  title="删除"
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

    <!-- 备注模态框 -->
    <div v-if="showNoteModal" class="modal">
      <div class="modal-content">
        <h3>备注内容</h3>
        <textarea 
          v-model="notingText" 
          class="edit-textarea"
          placeholder="请输入内容..."
        ></textarea>
        <div class="modal-actions">
          <button @click="cancelNote" class="btn btn-secondary">取消</button>
          <button @click="saveNote" class="btn btn-primary">保存</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { 
  BeakerIcon,
  Cog6ToothIcon,
  ArrowPathIcon,
  LockClosedIcon,
  StarIcon,
  ClipboardIcon,
  PencilSquareIcon,
  ClipboardDocumentListIcon,
  TrashIcon,
  Square2StackIcon
 } from '@heroicons/vue/24/outline'
import { 
  StarIcon as StarIconSolid
} from '@heroicons/vue/24/solid'

const router = useRouter()

// 响应式数据
const searchQuery = ref('')
const activeCategory = ref('all')
const showToast = ref(false)
const toastMessage = ref('')
const showEditModal = ref(false)
const showNoteModal = ref(false)
const editingIndex = ref(-1)
const editingText = ref('')
const notingIndex = ref(-1)
const notingText = ref('')
const test = ref('')

// 分类选项
const categories = ref([
  { id: 'all', name: '全部' },
  { id: 'image', name: '图片' },
  { id: 'video', name: '视频' },
  { id: 'file', name: '文件' },
  { id: 'favorite', name: '收藏' }
])

// 历史记录数据结构
const history = ref([])
const favoriteHistory = ref([])

// 过滤后的历史记录
const filteredHistory = computed(() => {
  let filtered = history.value
  
  // 搜索过滤 - 搜索内容和备注
  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase()
    filtered = filtered.filter(item => {
      const content = item.content ? item.content.toLowerCase() : ''
      const notes = item.notes ? item.notes.toLowerCase() : ''
      return content.includes(query) || notes.includes(query)
    })
  }
  
  
  // 分类过滤
  switch (activeCategory.value) {
    case 'image':
      filtered = filtered.filter(item => item.item_type === 'image')
      break
    case 'video':
      filtered = filtered.filter(item => item.item_type === 'video')
      break
    case 'file':
      filtered = filtered.filter(item => item.item_type === 'file')
      break
    case 'favorite':
      filtered = filtered.filter(item => item.is_favorite)
      break
    // 'all' 不进行过滤
  }
  
  return filtered
})

// 方法定义
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
const openSettings = async () => {
  router.push('/preferences')
  showMessage('打开设置')
}

// 刷新页面
const refreshPage = async () => {
  getAllHistory()
  showMessage('刷新成功')
}

// 添加到历史记录
const addToHistory = (text) => {
  if (!text.trim()) return
  
  const newItem = {
    content: text.trim(),
    timestamp: new Date().getTime(),
    is_favorite: false,
    item_type: 'text'
  }
  
  history.value.unshift(newItem)
  // 限制历史记录数量
  if (history.value.length > 100) {
    history.value.pop()
  }
}

const copyItem = async (item) => {
try {
  if (item.item_type === 'text') {
    // 对于文本类型，使用原来的文本复制方法
    await invoke('write_to_clipboard', { text: item.content });
    showToast('已复制文本');
  } else {
    // 对于文件和图片类型，使用新的文件复制方法
    await invoke('write_file_to_clipboard', { filePath: item.content });
    showToast(`已复制文件: ${getFileName(item.content)}`);
  }
} catch (error) {
  console.error('复制失败:', error);
  showToast(`复制失败: ${error}`);
}
}

// 切换收藏状态
const toggleFavorite = async (index) => {
  history.value[index].is_favorite = !history.value[index].is_favorite
  await invoke('set_favorite_status_by_id', { id: history.value[index].id })
  showMessage(history.value[index].is_favorite ? '已收藏' : '已取消收藏')
}

// 编辑项目
const editItem = (index) => {
  editingIndex.value = index
  editingText.value = history.value[index].content
  showEditModal.value = true
}

// 保存编辑
const saveEdit = async () => {
  if (editingIndex.value >= 0 && editingText.value.trim()) {
    history.value[editingIndex.value].content = editingText.value.trim()
    history.value[editingIndex.value].timestamp = new Date().getTime()
    await invoke('update_data_content_by_id', { 
      id: history.value[editingIndex.value].id, 
      newContent: editingText.value.trim() 
    })
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

// 备注项目
const noteItem = (index) => {
  notingIndex.value = index
  notingText.value = history.value[index].notes
  showNoteModal.value = true
}

// 保存备注
const saveNote = async () => {
  if (notingIndex.value >= 0 && notingText.value.trim()) {
    history.value[notingIndex.value].notes = notingText.value.trim()
    if (!notingText.value || notingText.value.trim() === '') {
      showMessage('内容不能为空')
    } else {
      await invoke('add_notes_by_id', { 
        id: history.value[notingIndex.value].id, 
        notes: notingText.value.trim() 
      })
    }
    showMessage('备注已更新')
  }
  cancelNote()
}

// 取消备注
const cancelNote = () => {
  showNoteModal.value = false
  notingIndex.value = -1
  notingText.value = ''
}

// 删除项目
const removeItem = async (index) => {
  await invoke('delete_data_by_id', { id: history.value[index].id })
  history.value.splice(index, 1)
  showMessage('已删除记录')
}

// 格式化时间
const formatTime = (timestamp) => {
  if (!timestamp) return '未知时间'
  
  const date = new Date(parseInt(timestamp))
  const now = new Date()
  const diff = now - date
  
  if (diff < 60000) return '刚刚'
  if (diff < 3600000) return `${Math.floor(diff / 60000)}分钟前`
  if (diff < 86400000) return `${Math.floor(diff / 3600000)}小时前`
  
  return date.toLocaleDateString()
}

const getAllHistory = async () => {
  try {
    const jsonString = await invoke('get_all_data')
    history.value = JSON.parse(jsonString)
    // 为现有数组中的每个对象添加 is_focus 字段
    history.value = history.value.map(item => ({
      ...item,
      is_focus: false
    }))
  } catch (error) {
    console.error('调用失败:', error)
  }
}

// 从路径中提取文件名
const getFileName = (path) => {
  if (!path) return '未知文件'
  return path.split(/[\\/]/).pop() || '未知文件'
}

// 图片加载错误处理
const handleImageError = (event) => {
  console.error('图片加载失败:', event.target.src)
}

// 检查是否是文档文件
const isDocumentFile = (path) => {
  if (!path) return false
  const docExtensions = ['.pdf', '.doc', '.docx', '.txt', '.md']
  return docExtensions.some(ext => path.toLowerCase().endsWith(ext))
}

// 生命周期
onMounted(async () => {
  console.log('开始初始化...')
  
  // 设置示例数据
  history.value = [
    {
      id: '0123456',
      item_type: 'text',        
      content: '这是一个测试样例',
      is_favorite: true,
      notes: '样例备注',
      timestamp: '1696118400000',
      is_focus: false
    }
  ]

  // 获取真实数据
  await getAllHistory()
  console.log('数据设置完成:', history.value)
  console.log('数据长度:', history.value.length)
})
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
  max-height: 100%;
}

/* 顶部搜索栏样式 */
.app-header {
  position: fixed; /* 新增：固定定位 */
  top: 0; /* 新增：固定在顶部 */
  left: 0; /* 新增：左侧对齐 */
  right: 0; /* 新增：右侧对齐 */
  z-index: 1000; /* 新增：确保在其他内容之上 */
  background: white;
  border-bottom: 1px solid #e1e8ed;
  padding: 0;
  max-width: 100%
}

.search-container {
  padding: 8px 10px;
  border-bottom: 1px solid #f0f0f0;
  -webkit-app-region: drag;
}

.search-bar {
  position: relative;
  margin: 0 auto;
  -webkit-app-region: no-drag;
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
  -webkit-app-region: drag;
}

.category-buttons {
  display: flex;
  gap: 0px;
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
  -webkit-app-region: no-drag;
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
  gap: 0px;
}

.icon-btn {
  padding: 4px;
  border: none;
  background: none;
  font-size: 18px;
  cursor: pointer;
  border-radius: 6px;
  transition: background 0.2s;
  -webkit-app-region: no-drag;
}

.icon-btn:hover {
  background: #e9ecef;
}

.icon-settings {
  width: 1.2rem;
  height: 1.2rem;
  position: relative;
  top: 3px; 
  color: #595959;
}

.icon-settings:hover {
  width: 1.2rem;
  height: 1.2rem;
  position: relative;
  top: 3px; 
  color: #3282f6;
}

.icon-default {
  width: 1rem;
  height: 1rem;
  position: relative;
  top: 3px; 
  color: #595959;
}

.icon-default:hover {
  width: 1rem;
  height: 1rem;
  position: relative;
  top: 3px; 
  color: #3282f6;
}

.icon-star-solid {
  width: 1rem;
  height: 1rem; 
  position: relative;
  top: 3px; 
  color: #f1c40f;
}
/* 主内容区样式 */
.app-main {
  padding: 8px 10px;
  margin: 0 auto;
  margin-top: 96px; /* 顶部搜索栏高度 + 工具栏高度 */
  overflow-x: hidden;
  max-width: 100%;
}

/* 空状态样式 */
.empty-state {
  text-align: center;
  padding: 60px 20px;
  color: #7f8c8d;
  max-width: 100%;
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
  max-width: 100%;
}

.history-item {
  background: white;
  border: 1px solid #e1e8ed;
  border-radius: 12px;
  padding: 2px 5px;
  transition: all 0.2s ease;
  position: relative;
  max-width: 100%;
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

/* 剪贴文本样式 */
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

/* 剪贴图片预览样式 */
.image-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
}

.preview-image {
  max-width: 100%;
  max-height: 100%;
  border-radius: 4px;
  object-fit: contain;
}

.image-filename {
  font-size: 12px;
  color: #666;
  text-align: center;
}

/* 剪贴文件预览样式 */
.file-container {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 8px;
  border: 1px solid #e0e0e0;
  border-radius: 6px;
  background-color: #f9f9f9;
}

.file-icon {
  font-size: 24px;
}

.file-info {
  flex: 1;
  min-width: 0; /* 允许文本截断 */
}

.file-name {
  font-weight: 500;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.file-path {
  font-size: 12px;
  color: #888;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
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

/* 淡入淡出动画效果 */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.1s ease, transform 0.1s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(-10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

/* 响应式设计 */
@media (max-width: 768px) {
  .toolbar {
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
}
</style>