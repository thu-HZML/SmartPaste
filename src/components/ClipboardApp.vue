<template>
  <div class="app" @mousedown="startDragging">
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
          <button class="icon-btn" @click="showDeleteAll">           
            <TrashIcon class="icon-settings" />
          </button>
        </div>
      </div>
    </header>

    <!-- 剪贴板记录列表 -->
    <main class="app-main">
      <!-- "全部"、"图片"、"视频"、"文件"、"收藏夹内容"界面 -->
      <div v-if="['all', 'image', 'video', 'file', 'folder'].includes(activeCategory)">
        <div v-if="filteredHistory.length === 0" class="empty-state">
          <p v-if="searchQuery">未找到匹配的记录</p>
          <p v-else>暂无剪贴板记录</p>
          <p class="hint">复制的内容将显示在这里</p>
        </div>
        
        <div v-else class="history-list-reverse">
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
                <span v-if = "item.item_type === 'text'">{{ item.size }}字符</span>
                <span v-else>{{ formatFileSize(item.size) }}</span>
                <span>{{ formatTime(item.timestamp) }}</span>
              </div>

              <!-- 右上方按钮组 -->
              <div class="item-actions-top">
                <button 
                  v-if="item.item_type === 'image'"
                  class="icon-btn-small" 
                  @click="showOCR(item)"
                  title="图片转文字"
                >
                  <span class="content-OCR">{{ 'OCR' }}</span>
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="toggleFavorite(item)"
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
                  @click="editItem(item)"
                  title="编辑"
                >
                  <ClipboardIcon class="icon-default" />
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="noteItem(item)"
                  title="备注"
                >
                  <PencilSquareIcon class="icon-default" />
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="removeItem(item)"
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
                      <img 
                        :src="item.iconData"
                        class="file-icon"
                        @error="handleIconError"
                      />
                      <div class="file-name">{{ getFileName(item.content) }}</div>
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
        <div class="history-list">
          <!-- 新建收藏夹 -->
          <div class="folder-item" @click="showFolder()">
            <div class="folder-content">
              <FolderPlusIcon class="icon-folder" />
              <span class="folder-name">{{ '新建收藏夹' }}</span>                        
            </div>
          </div>
          <!-- 普通收藏夹 -->
          <div 
            v-for="(item, index) in folders" 
            :key="index" 
            class="folder-item"
            tabindex="0"
            @click="showFolderContent(item)"
          >
            <div class="folder-content">
              <FolderIcon class="icon-folder" />
              <span class="folder-name" :title="item.name">{{ item.name }}</span>
              <span class="content-count">{{ item.num_items }}</span> 
              <button 
                class="icon-btn-small" 
                @click.stop="noteItem(item)"
                title="重命名"
              >
                <PencilSquareIcon class="icon-default" />
              </button>
              <button 
                class="icon-btn-small" 
                @click.stop="removeFolder(item)"
                title="删除"
              >
                <TrashIcon class="icon-default" />
              </button>             
            </div>
          </div>
        </div>
      </div>
    </main>

    <!-- 操作提示 -->
    <div v-if="showToast" class="toast">
      {{ toastMessage }}
    </div>

    <!-- 删除提醒模态框 -->
    <div v-if="showDeleteModal" class="modal">
      <div class="modal-content">
        <h3>确定要清空所有未收藏的历史记录吗？此操作不可撤销！</h3>
        <div class="modal-actions-center">
          <button @click="cancelDeleteAll" class="btn btn-secondary">取消</button>
          <button @click="deleteAllHistory" class="btn btn-least">删除</button>
        </div>
      </div>
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

    <!-- OCR模态框 -->
    <div v-if="showOcrModal" class="modal">
      <div class="modal-content">
        <h3>图片转文字内容</h3>
        <textarea 
          v-model="ocrText" 
          class="edit-textarea"
          placeholder="请输入内容..."
        ></textarea>
        <div class="modal-actions">
          <button @click="cancelOCR" class="btn btn-secondary">取消</button>
          <button @click="copyOCR" class="btn btn-primary">复制</button>
        </div>
      </div>
    </div>

    <!-- 新建收藏夹模态框 -->
    <div v-if="showFolderModal" class="modal">
      <div class="modal-content">
        <h3>收藏夹名称</h3>
        <textarea 
          v-model="folderNotingText" 
          class="edit-textarea"
          placeholder="请输入内容..."
        ></textarea>
        <div class="modal-actions">
          <button @click="cancelFolder" class="btn btn-secondary">取消</button>
          <button @click="addFolder" class="btn btn-primary">创建</button>
        </div>
      </div>
    </div>

    <!-- 历史记录添加至收藏夹模态框 -->
    <div v-if="showFoldersModal" class="modal">
      <div class="modal-content">
        <h3>添加到收藏夹</h3>
        <div class="folders-container">
          <div class="history-list">      
            <!-- 普通收藏夹 -->
            <div 
              v-for="(item, index) in folders" 
              :key="index" 
              class="folder-item-toast"
              tabindex="0"
              @click="selectFolder(item)"        
            >
              <div class="folder-content-toast">
                <div class="custom-folder-icon" :class="{ 'selected': item.isSelected }"></div>
                <span class="folder-name" :title="item.name">{{ item.name }}</span>
                <span class="content-count">{{ item.numItems }}个内容</span>                      
              </div>
            </div>

            <!-- 新建收藏夹 -->
            <div class="search-bar">           
              <input 
                type="text" 
                v-model="folderQuery"
                placeholder="新建收藏夹：请输入名称" 
                class="toast-input"
              >
              <button @click="addFolderToast" class="btn-create">创建</button>
            </div>  
          </div>       
        </div>       
        <div class="modal-actions">
          <button @click="cancelAddToFolder" class="btn btn-secondary">取消</button>
          <button @click="addToFolder" class="btn btn-primary">确认</button>
        </div>
      </div>    
    </div>
  </div>
</template>

<script setup>
import { ref, computed, onMounted, onUnmounted, watch } from 'vue'
import { useRouter } from 'vue-router'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window';
import { toggleClipboardWindow } from '../utils/actions.js'
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
  Square2StackIcon,
  FolderPlusIcon,
  FolderIcon
 } from '@heroicons/vue/24/outline'
import { 
  StarIcon as StarIconSolid
} from '@heroicons/vue/24/solid'

const router = useRouter()
const currentWindow = getCurrentWindow();

// 响应式数据
const searchQuery = ref('')
const activeCategory = ref('all')
const showToast = ref(false)
const toastMessage = ref('')
const showEditModal = ref(false)
const showNoteModal = ref(false)
const showFolderModal = ref(false)
const showFoldersModal = ref(false)
const showOcrModal = ref(false)
const showDeleteModal = ref(false)
const editingText = ref('')
const editingItem = ref(null)
const notingText = ref('')
const notingItem = ref(null)
const ocrText = ref('')
const folderNotingText = ref('')
const currentFolder = ref(null)
const searchLoading = ref(false)
const currentItem = ref(null)
const folderQuery = ref('')
const unlistenFocusChanged = ref(null) // 存储取消监听的函数
const test = ref('')

// 防抖定时器
let searchTimeout = null

// 双击定时器
let clickTimeout = null

// 存储是否在拖动
let isDragging = null

// 分类选项
const categories = ref([
  { id: 'all', name: '全部' },
  { id: 'image', name: '图片' },
  { id: 'video', name: '视频' },
  { id: 'file', name: '文件' },
  { id: 'favorite', name: '收藏' }
])

// 历史记录数据结构
const folders = ref([])
const filteredHistory = ref([])
const initialSelectedFolders = ref([]) // 存储当前记录被收藏进的收藏夹
const iconCache = ref({}) // 用于缓存已加载的图标

/*
// 计算属性
const displayHistory = computed(() => {
  if (activeCategory.value === 'folder') {
    return folders.value
  } else {
    return filteredHistory.value
  }
})
*/

// 监听 searchQuery 变化
watch(searchQuery, async(newQuery) => {
  await handleSearch(newQuery)
})

// 监听 activeCategory 变化
watch(activeCategory, async (currentCategory) => {
  await handleCategoryChange(currentCategory)
})

// 搜索逻辑
const handleSearch = async (query) => {
  // 清除之前的定时器
  clearTimeout(searchTimeout)
  
  // 空查询立即返回
  if (query.trim() === '') {
    await getAllHistory()
    searchLoading.value = false
    return
  }
  
  searchLoading.value = true
  
  // 设置新的定时器（300ms 防抖）
  searchTimeout = setTimeout(async () => {
    await performSearch(query)
  }, 300)
}

// 分类逻辑
const handleCategoryChange = async (category) => {
  if (['image', 'video', 'file'].includes(category)) {
    searchLoading.value = true
    await performClassify(category)
    return
  }
  else if (category.trim() === 'all'){
    searchLoading.value = true
    await getAllHistory()
  }
  else if (category.trim() === 'favorite'){
    searchLoading.value = true
    console.log('默认收藏夹内容个数：', folders.value[0].num_items)
    await getAllFolders()
  }
  else if (category.trim() === 'folder') {
    await performFolder()
    return
  }
}

// 搜索过滤
const performSearch = async (query) => { 
  try {
    const result = await invoke('search_text_content', { 
      query: query.trim() 
    })
    
    filteredHistory.value = JSON.parse(result)
  } catch (err) {
    console.error('搜索失败:', err)
  } finally {
    searchLoading.value = false
  }
}

// 分类过滤
const performClassify = async (currentCategory) => { 
  try {
    const result = await invoke('filter_data_by_type', { 
      itemType: currentCategory.trim() 
    })    
    filteredHistory.value = JSON.parse(result)
  } catch (err) {
    console.error('分类失败:', err)
  } finally {
    searchLoading.value = false
  }
}

// 收藏夹过滤
const performFolder = async () => { 
  console.log('开始筛选收藏夹')
  if (currentFolder.value.name === '默认收藏夹') {
    console.log('进入默认收藏夹')
    try {
      const result = await invoke('filter_data_by_favorite', { 
        isFavorite: true
      })    
      filteredHistory.value = JSON.parse(result)
      console.log('收藏夹数量:', folders.value[0].num_items)
    } catch (err) {
      console.error('默认收藏夹获取失败:', err)
    }
  }
  else {
    console.log('进入收藏夹：', currentFolder.value.name)
    try {
      const result = await invoke('filter_data_by_folder', { 
        folderName: currentFolder.value.name
      })    
      filteredHistory.value = JSON.parse(result)
    } catch (err) {
      console.error('获取收藏夹内容失败:', err)
    } finally {
      searchLoading.value = false
    }
  }
}

// 消息弹窗
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
  // 移除窗口焦点监听器
  removeWindowListeners()

  router.push('/preferences')
  showMessage('打开设置')
}

// 复制历史内容
const copyItem = async (item) => {
  try {
    if (item.item_type === 'text') {
      // 对于文本类型，使用原来的文本复制方法
      await invoke('write_to_clipboard', { text: item.content })
      showMessage('已复制文本')
    } else {
      // 对于文件和图片类型，使用新的文件复制方法
      await invoke('write_file_to_clipboard', { filePath: item.content })
      showMessage(`已复制文件: ${getFileName(item.content)}`)
    }
  } catch (error) {
    console.error('复制失败:', error)
    showMessage(`复制失败: ${error}`)
  }
}

// 切换收藏状态
const toggleFavorite = async (item) => {
  // 清除之前的单击定时器
  if (clickTimeout || activeCategory.value === 'folder') {
    clearTimeout(clickTimeout)
    // 如果已经有定时器存在，说明是双击
    executeDoubleClick(item)
    return;
  }
  
  // 设置新的定时器
  clickTimeout = setTimeout(async () => {
    // 定时器触发，说明是单击
    item.is_favorite = !item.is_favorite
    await invoke('set_favorite_status_by_id', { id: item.id })
    showMessage(item.is_favorite ? '已收藏' : '已取消收藏')
    clickTimeout = null;
  }, 150); // 150ms内再次点击视为双击
}

// 双击弹出收藏夹选择
const executeDoubleClick = async (item) => {
    showMessage('执行了双击操作')
    showFoldersModal.value = true
    currentItem.value = item
    // 清除定时器
    clickTimeout = null;

    // 选中所有已有该记录的收藏夹
    try {
      const foldersString = await invoke('get_folders_by_item_id', { itemId: item.id })
      const foldersJson = JSON.parse(foldersString)

      // 保存初始选中状态
      initialSelectedFolders.value = foldersJson.map(f => f.id)

      folders.value = folders.value.map(folder => {
        // 检查当前文件夹是否在foldersJson中（即包含该项目）
        const isContained = foldersJson.some(f => f.id === folder.id)

        return {
          ...folder,
          isSelected: isContained
        }
      })
      
      // 如果项目已被收藏但不在任何收藏夹中，确保默认收藏夹被选中
      if (item.is_favorite) {
        const defaultFolder = folders.value.find(folder => folder.name === '默认收藏夹')
        if (defaultFolder && !defaultFolder.isSelected) {
          defaultFolder.isSelected = true
        }
      }
    } catch(err) {
      console.error('获取收藏夹失败:', err)
    }
}

// 弹出"确认删除"提示框
const showDeleteAll = () => {
  showDeleteModal.value = true
}

// 删除所有历史记录
const deleteAllHistory = async () => {
  try {
    await invoke('delete_unfavorited_data')
    showMessage('已清除所有未收藏记录')
    handleSearch(searchQuery.value)
    handleCategoryChange(activeCategory.value)
  } catch(err) {
    console.error('清除历史记录失败:', err)
  }
  cancelDeleteAll()
}

// "确认删除"提示框消失
const cancelDeleteAll = () => {
  showDeleteModal.value = false
}

// 编辑项目
const editItem = (item) => {
  editingItem.value = item
  editingText.value = item.content
  showEditModal.value = true
}

// 保存编辑
const saveEdit = async () => {
  if (editingText.value.trim() && editingItem) {
    editingItem.value.content = editingText.value.trim()
    editingItem.value.timestamp = new Date().getTime()
    await invoke('update_data_content_by_id', { 
      id: editingItem.value.id,
      newContent: editingText.value.trim() 
    })
    showMessage('内容已更新')
  }
  cancelEdit()
}

// 取消编辑
const cancelEdit = () => {
  showEditModal.value = false
  editingItem.value = null
  editingText.value = ''
}

// 备注项目
const noteItem = (item) => {
  notingItem.value = item
  notingText.value = item.notes
  showNoteModal.value = true
}

// 保存备注
const saveNote = async () => {
  if (notingText.value.trim() && notingItem) {
    notingItem.value.notes = notingText.value.trim()
    if (!notingText.value || notingText.value.trim() === '') {
      showMessage('内容不能为空')
    } else {
      await invoke('add_notes_by_id', { 
        id: notingItem.value.id, 
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
  notingItem.value = null
  notingText.value = ''
}

// 显示OCR内容
const showOCR = async (item) => {
  const ocrString = await invoke('ocr_image', { filePath: item.content })
  console.log(ocrString)
  ocrText.value = JSON.parse(ocrString)[0].text
  showOcrModal.value = true
}

// 复制OCR内容
const copyOCR = async () => {
  if (!ocrText.value || ocrText.value.trim() === '') {
    showMessage('内容不能为空')
  } else {
    // 添加到数据库
    await invoke('insert_received_text_data', { text: ocrText.value })
    // 复制该内容
    await invoke('write_to_clipboard', { text: ocrText.value })
    showMessage('已复制OCR内容')

    // 刷新界面
    handleSearch(searchQuery.value)
    handleCategoryChange(activeCategory.value)
  }
  cancelOCR()
}

// 关闭OCR弹窗
const cancelOCR = () => {
  showOcrModal.value = false
  ocrText.value = ''
}

// 删除历史记录
const removeItem = async (item) => {
  try {
    // 如果记录被收藏，先从所有收藏夹中移除
    if (item.is_favorite) {
      // 获取包含该记录的所有收藏夹
      const foldersString = await invoke('get_folders_by_item_id', { itemId: item.id })
      const foldersContainingItem = JSON.parse(foldersString)
      
      // 从每个收藏夹中移除该记录
      const removePromises = foldersContainingItem.map(folder => 
        invoke('remove_item_from_folder', {
          folderId: folder.id,
          itemId: item.id
        })
      )
      
      await Promise.all(removePromises)
    }
    
    // 删除历史记录本身
    await invoke('delete_data_by_id', { id: item.id })
    
    // 从前端列表中移除
    const index = filteredHistory.value.findIndex(i => i.id === item.id)
    if (index !== -1) {
      filteredHistory.value.splice(index, 1)
    }
    
    showMessage('已删除记录')
  } catch (error) {
    console.error('删除记录失败:', error)
    showMessage('删除记录失败')
  }
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

// 格式化文件大小
const formatFileSize = (bytes) => {
  if (bytes === 0 || !bytes) return '0 B'
  
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const base = 1024
  
  // 处理边界情况
  if (bytes < base) {
    return `${bytes} B`
  }
  
  const exponent = Math.min(Math.floor(Math.log(bytes) / Math.log(base)), units.length - 1)
  const size = (bytes / Math.pow(base, exponent)).toFixed(1)
  
  // 移除 .0 后缀
  const cleanSize = size.endsWith('.0') ? size.slice(0, -2) : size
  
  return `${cleanSize} ${units[exponent]}`
}

// 获取所有历史记录
const getAllHistory = async () => {
  try {
    const jsonString = await invoke('get_all_data')
    filteredHistory.value = JSON.parse(jsonString)
    // 为现有数组中的每个对象添加 is_focus 字段
    filteredHistory.value = filteredHistory.value.map(item => ({
      ...item,
      is_focus: false,
      iconData: null // 初始化为null
    }))

    // 为文件类型的项目加载图标
    await loadIconsForFiles()
  } catch (error) {
    console.error('调用失败:', error)
  }
}

// 获取所有收藏夹
const getAllFolders = async () => {
  try {
    const jsonString = await invoke('get_all_folders')
    folders.value = JSON.parse(jsonString)

    // 创建默认收藏夹
    if (!folders.value || folders.value.length === 0) {
      console.log('收藏夹为空，正在创建默认收藏夹...')
      try {
        await invoke('create_new_folder', { name: '默认收藏夹' })
        console.log('默认收藏夹创建成功')
        
        // 重新获取收藏夹列表
        const updatedJsonString = await invoke('get_all_folders')
        folders.value = JSON.parse(updatedJsonString)
        console.log('更新后的收藏夹:', folders.value)
      } catch (createError) {
        console.error('创建默认收藏夹失败:', createError)
      }
    }

    folders.value = folders.value.map(item => ({
      ...item,
      isSelected: false
    }))
  } catch (error) {
    console.error('get_all_folders调用失败:', error)
  }
}

// 从路径中提取文件名
const getFileName = (path) => {
  if (!path) return '未知文件'

  // 提取文件名
  const fileName = path.split(/[\\/]/).pop() || '未知文件'
  
  // 使用正则表达式移除时间戳前缀（数字+连字符）
  return fileName.replace(/^\d+-/, '') || '未知文件'
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

// 显示创建收藏夹模态框
const showFolder = () => {
  showFolderModal.value = true
}

// 创建收藏夹
const addFolder = async () => {
  try {
    await invoke('create_new_folder', { name: folderNotingText.value.trim() })
    getAllFolders()
  } catch (err) {
    console.error('创建文件夹失败', err)
  }
  showMessage('新收藏夹已创建')
  cancelFolder()
}

// 取消创建收藏夹
const cancelFolder = () => {
  showFolderModal.value = false
  folderNotingText.value = ''
}

// 删除收藏夹
const removeFolder = async (item) => {
  await invoke('delete_folder', { folderId: item.id })
  const index = folders.value.findIndex(i => i.id === item.id)
  if (index !== -1) {
    folders.value.splice(index, 1)
  }
  showMessage('已删除收藏夹')
}

// 模态框创建收藏夹
const addFolderToast = async () => {
  try {
    // 空查询立即返回
    if (folderQuery.value === '') {
      showMessage('收藏夹名称不能为空！')
      return
    }

    await invoke('create_new_folder', { name: folderQuery.value })
    getAllFolders()
    folderQuery.value = ''
  } catch (err) {
    console.error('创建文件夹失败', err)
  }
  showMessage('新收藏夹已创建')
}

// 显示收藏夹内容
const showFolderContent = async (item) => {
  activeCategory.value = 'folder'
  currentFolder.value = item
}

// 切换收藏夹选中状态
const selectFolder = (item) => {
  // 如果是默认收藏夹
  if (item.name === '默认收藏夹') {
    // 如果取消选中默认收藏夹，则取消所有其他收藏夹
    if (item.isSelected) {
      // 取消选中默认收藏夹
      item.isSelected = false
      // 取消所有其他收藏夹的选中
      folders.value.forEach(folder => {
        if (folder.name !== '默认收藏夹') {
          folder.isSelected = false
        }
      })
    } else {
      // 选中默认收藏夹
      item.isSelected = true
    }
  } else {
    // 非默认收藏夹
    // 切换当前项的选中状态
    item.isSelected = !item.isSelected
    
    // 如果选中了任何非默认收藏夹，确保默认收藏夹也被选中
    if (item.isSelected) {
      const defaultFolder = folders.value.find(folder => folder.name === '默认收藏夹')
      if (defaultFolder && !defaultFolder.isSelected) {
        defaultFolder.isSelected = true
      }
    }
  }
}

// 把历史记录添加到收藏夹中
const addToFolder = async () => {
  try {
    const selectedFolders = folders.value.filter(item => 
      item.isSelected && item.name !== '默认收藏夹'
    )
    const previouslySelectedFolders = folders.value.filter(item => 
      initialSelectedFolders.value.includes(item.id) && !item.isSelected
    )
    
    // 并行处理所有选中的文件夹
    const addPromises = selectedFolders.map(item => 
      invoke('add_item_to_folder', { 
        folderId: item.id,
        itemId: currentItem.value.id
      })
    )   
    
    // 从之前选中但现在未选中的收藏夹中移除
    const removePromises = previouslySelectedFolders.map(item =>
      invoke('remove_item_from_folder', {
        folderId: item.id,
        itemId: currentItem.value.id
      })
    )

    await Promise.all([...addPromises, ...removePromises])
    showMessage('已收藏进指定文件夹')

    if (folders.value[0].isSelected) {
      currentItem.value.is_favorite = true
      await invoke('favorite_data_by_id', { id: currentItem.value.id })
    } else {
      currentItem.value.is_favorite = false
      await invoke('unfavorite_data_by_id', { id: currentItem.value.id })
    }

    // 刷新界面
    handleSearch(searchQuery.value)
    handleCategoryChange(activeCategory.value)
  } catch (err) {
    console.error('创建文件夹失败', err)
  }
  cancelAddToFolder()
}

// 取消添加至收藏夹
const cancelAddToFolder = () => {
  showFoldersModal.value = false
  folders.value = folders.value.map(item => ({
    ...item,
    isSelected: false
  }))

  initialSelectedFolders.value = [] // 重置初始选中状态
}

// 主窗口监听剪贴板事件
const setupClipboardRelay = async () => {
  const unlisten = await listen('clipboard-updated', async (event) => {
    console.log('接受后端更新消息')
    console.log('通过中转收到剪贴板事件:', event.payload)
    // 刷新历史记录
    handleSearch(searchQuery.value)
    handleCategoryChange(activeCategory.value)
  })
  
  return unlisten
}

// 新增：为文件类型的项目加载图标
const loadIconsForFiles = async () => {
  const fileItems = filteredHistory.value.filter(item => item.item_type === 'file')
  
  for (const item of fileItems) {
    try {
      // 如果缓存中已有，直接使用
      if (iconCache.value[item.content]) {
        item.iconData = iconCache.value[item.content]
      } else {
        const iconBase64 = await loadIcon(item.content)
        if (iconBase64) {
          item.iconData = iconBase64
          iconCache.value[item.content] = iconBase64 // 缓存图标
        }
      }
    } catch (error) {
      console.error('加载图标失败:', error)
      item.iconData = null
    }
  }
}

// 修改 loadIcon 函数，确保返回正确的数据
const loadIcon = async (filePath) => {
  if (!filePath) return null
  
  try {
    const iconBase64 = await invoke('get_file_icon', { path: filePath });
    console.log("获取图标成功:", filePath);
    return iconBase64;
  } catch (error) {
    console.error("获取图标失败:", error, filePath);
    return null;
  }
}

// 使用 Tauri API 开始拖动窗口
const startDragging = async (event) => {
  // 防止在输入框上触发拖动
  if (event.target.tagName === 'INPUT' || event.target.closest('input')) {
    return
  }
  
  // 防止在按钮上触发拖动
  if (event.target.tagName === 'BUTTON' || event.target.closest('button')) {
    return
  }
  
  // 防止在图标上触发拖动
  if (event.target.tagName === 'svg' || event.target.tagName === 'path' || event.target.closest('svg')) {
    return
  }
  
  // 防止在模态框上触发拖动
  if (event.target.closest('.modal')) {
    return
  }
  
  try {
    isDragging = true
    await currentWindow.startDragging()
  } catch (error) {
    console.error('开始拖动失败:', error)
  } finally {
    // 使用 setTimeout 确保拖动操作完成后再重置状态
    setTimeout(() => {
      isDragging = false
    }, 100)
  }
}

// 窗口失焦时自动关闭窗口
const setupWindowListeners = async () => { 
  // 如果已经存在监听器，先移除
  if (unlistenFocusChanged.value) {
    unlistenFocusChanged.value()
    unlistenFocusChanged.value = null
  }

  // 监听窗口失去焦点事件
  unlistenFocusChanged.value = await currentWindow.onFocusChanged(async ({ payload: focused }) => {
    if (!focused) {   
      if (isDragging) {
        console.log('检测到正在拖动，不关闭窗口')
        return
      }
      console.log('窗口失去焦点，准备关闭')
      currentWindow.close()
    }
    else {
      console.log('窗口获得焦点')
    }
  })
}

// 移除窗口失焦监听函数
const removeWindowListeners = () => {
  if (unlistenFocusChanged.value) {
    unlistenFocusChanged.value()
    unlistenFocusChanged.value = null
    console.log('已移除窗口焦点监听器')
  }
}

// 生命周期
onMounted(async () => {
  console.log('开始初始化...')

  // OCR配置
  await invoke('configure_ocr', {})
  
  // 开启后端监听
  await setupClipboardRelay()
  
  // 设置窗口事件监听器
  await setupWindowListeners()
  
  // 设置窗口聚焦
  currentWindow.setFocus()

  // 初始化窗口大小
  try {
    await currentWindow.setSize(new LogicalSize(400, 600));
  } catch (error) {
    console.error('设置窗口大小失败:', error)
  }

  // 设置示例数据
  filteredHistory.value = [
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

  // 获取历史记录
  await getAllHistory()

  // 获取收藏夹记录
  await getAllFolders()
  console.log('数据长度:', filteredHistory.value.length)
 
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
}

.search-bar {
  display: flex;
  flex-direction: row;
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
  align-items: center;
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
  top: 1px; 
  color: #595959;
}

.icon-default:hover {
  width: 1rem;
  height: 1rem;
  position: relative;
  top: 1px; 
  color: #3282f6;
}

.icon-star-solid {
  width: 1rem;
  height: 1rem; 
  position: relative;
  top: 1px; 
  color: #f1c40f;
}

.icon-folder {
  width: 2rem;
  height: 2rem;
  position: relative; 
  color: #595959;
}

/* OCR标签 */
.content-OCR {
  border: 1px solid #3282f6;
  border-radius: 3px;
  padding: 1px;
  color: #595959;
  flex-shrink: 0;
  margin-left: auto;
}

.content-OCR:hover {
  color: #3282f6;
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

/* 历史记录列表样式（倒序） */
.history-list-reverse {
  display: flex;
  flex-direction: column-reverse;
  gap: 12px;
  max-width: 100%;
}

/* 历史记录列表样式（正序） */
.history-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
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
  align-items: flex-start;
  gap: 16px;
  margin-bottom: 10px;
}

.item-text {
  display: -webkit-box;
  line-clamp: 4;          /* 限制显示行数 */
  -webkit-line-clamp: 4;      /* 限制显示行数 */
  white-space: pre-wrap;  /* 保留连续空格和换行 */
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  font-size: 14px;
  line-height: 1.5;
  word-break: break-word;
  color: #1f1f1f;
  min-height: 83px;
  max-height: 83px;
  align-items: center;
}

/* 剪贴图片预览样式 */
.image-container {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  width: 100%;
  max-height: 150px;
  overflow: hidden;
}

.preview-image {
  max-width: 100%;
  max-height: 65px;
  width: auto;
  height: auto;
  border-radius: 4px;
  object-fit: contain; /* 保持比例，完整显示图片 */
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
  gap: 8px;
  overflow: hidden;
  height: 80px;
}

.file-icon {
  max-height: 50px;
}

.file-name {
  display: -webkit-box;
  line-clamp: 2;          /* 限制显示行数 */
  -webkit-line-clamp: 2;      /* 限制显示行数 */
  white-space: pre-wrap;  /* 保留连续空格和换行 */
  -webkit-box-orient: vertical;
  overflow: hidden;
  text-overflow: ellipsis;
  flex: 1;
  font-size: 14px;
  line-height: 1.5;
  word-break: break-word;
  color: #1f1f1f;
  max-height: 42px;
}

/* 收藏夹样式 */
.folder-item {
  background: white;
  border: 1px solid #e1e8ed;
  border-radius: 8px;
  padding: 2px 5px;
  transition: all 0.2s ease;
  position: relative;
  max-width: 100%;
  font-size: 20px;
  color: #595959;
}

.folder-item:hover {
  border-color: #b7c8fe;
}

.folder-item:focus {
  border-color: #3282f6;
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.1);
}

.folder-content {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 8px;
  margin-bottom: 10px;
  height: 40px;
}

.folder-content:hover,
.folder-content:hover .icon-folder,
.folder-content:hover .folder-name,
.folder-content:hover .content-count {
  color: #3282f6;
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
  z-index: 10000;
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

.modal-actions-center {
  display: flex;
  gap: 12px;
  justify-content: center;
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

.btn-least {
  background: #d24d15;
  color: white;
}

.btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(0,0,0,0.15);
}

.btn-create {
  padding: 10px 16px;
  border: none;
  border-radius: 2px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  color: #416afe;
  background: #e4edfd;
}

/* 收藏夹容器 - 限制高度并添加滚动 */
.folders-container {
  max-height: 300px; /* 设置最大高度 */
  overflow-y: auto; /* 添加垂直滚动 */
  overflow-x: hidden;
  margin-bottom: 16px; /* 与下方元素间距 */
}

/* 收藏夹样式 */
.folder-item-toast {
  border: none;
  background: none;
  padding: 0px 5px;
  transition: all 0.2s ease;
  position: relative;
  max-width: 100%;
  font-size: 20px;
  color: #595959;
}

.folder-item-toast:hover {
  color: #b7c8fe;
}

/* 自定义图标样式 */
.custom-folder-icon {
  width: 20px;
  height: 20px;
  border-radius: 4px;
  border: 1px solid #d9d9d9;
  display: flex;
  align-items: center;
  justify-content: center;
  position: relative;
  transition: all 0.3s ease;
  flex-shrink: 0;
}

/* 对勾样式 */
.custom-folder-icon::after {
  content: "";
  position: absolute;
  width: 9px;
  height: 4px;
  border-left: 2px solid transparent;
  border-bottom: 2px solid transparent;
  transform: rotate(-45deg);
  transition: all 0.3s ease;
}

/* 悬停时图标效果 */
.folder-item-toast:hover .custom-folder-icon {
  border-color: #b7c8fe;
}

/* 选中状态 - 蓝色背景和白色对勾 */
.custom-folder-icon.selected {
  background-color: #3498db;
  border-color: #3498db;
}

.custom-folder-icon.selected::after {
  border-left-color: white;
  border-bottom-color: white;
}

.folder-content-toast {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  margin-bottom: 10px;
  font-size: 15px;
}

.folder-name {
  flex: 1;
  text-align: left;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.content-count {
  color: #8c8c8c;
  flex-shrink: 0;
  margin-left: auto;
}

/* 输入框样式 */
.toast-input {
  width: 80%;
  padding: 6px 10px 6px 23px;
  border: 1px solid #e1e8ed;
  border-radius: 2px;
  font-size: 16px;
  outline: none;
  transition: all 0.2s;
}

.toast-input:hover {
  border-color: #b7c8fe;
}

.toast-input:focus {
  border-color: #3282f6;
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.1);
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