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
          <input 
            type="datetime-local" 
            v-model="startTime"
            class="time-input"
          >
          <input 
            type="datetime-local" 
            v-model="endTime"
            class="time-input"
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
          <button 
            v-if="activeCategory === 'folder'" 
            :class="['category-btn', { active: true }]"
          >
            内容
          </button>
        </div>
        
        <div class="toolbar-actions">
          <!-- 多选复制按钮 -->
          <button 
            v-if="showMultiCopyBtn" 
            class="icon-btn" 
            @click="copySelectedItems"
            title="复制选中的项目"
          >
            <Square2StackIconSolid class="icon-settings" />
          </button>
          <!-- 固定视图按钮 -->
          <button class="icon-btn" @click="togglePinnedView">
            <LockOpenIcon v-if="canDeleteWindow" class="icon-settings" />
            <LockClosedIcon v-else class="icon-settings" />
          </button>
          <!-- 打开设置按钮 -->
          <button class="icon-btn" @click="openSettings">         
            <Cog6ToothIcon class="icon-settings" />
          </button>
          <!-- 清空按钮 -->
          <button class="icon-btn" @click="showDeleteAll">           
            <TrashIcon class="icon-settings" />
          </button>
        </div>
      </div>
    </header>

    <!-- 剪贴板记录列表 -->
    <main class="app-main">
      <!-- "全部"、"图片"、"视频"、"文件"、"收藏夹内容"界面 -->
      <div v-if="['all', 'text', 'image', 'file', 'folder'].includes(activeCategory)">
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
            :class="{ 'selected': item.is_selected }"
            tabindex="0"
            @click="handleItemClick(item, $event)"
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
                <div v-if="item.selectionOrder" class="selection-order-badge">
                  {{ item.selectionOrder }}
                </div>
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
                  <PencilSquareIcon class="icon-default" />
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="noteItem(item)"
                  title="备注"
                >
                  <ClipboardDocumentListIcon class="icon-default" />
                </button>
                <button 
                  class="icon-btn-small" 
                  @click="showDeleteSingle(item)"
                  title="删除"
                >
                  <TrashIcon class="icon-default" />
                </button>
              </div>
            </div>
            <div class="item-content">            
              <div v-if="item.is_focus || !item.notes" class="item-text">
                <!-- 显示文本 -->
                <div v-if="item.item_type === 'text'" :title="item.content">
                  {{ item.content }}
                </div>
                
                <!-- 显示图片 -->
                <div v-else-if="item.item_type === 'image'" class="image-container">
                  <img 
                    v-if="item.content"
                    :src="convertFileSrc(normalizedPath + item.content)" 
                    :alt="'图片: ' + getFileName(item.content)"
                    class="preview-image"
                    @error="handleImageError"
                  />
                  <div v-else class="loading">加载中...</div>
                  <div class="image-filename">{{ getFileName(item.content) }}</div>
                </div>

                    <!-- 显示文件 -->
                    <div v-else-if="['file', 'folder'].includes(item.item_type)" class="file-container">
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
                <ClipboardDocumentListIcon class="icon-notes" />
                {{ item.notes }}
              </div>
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
          >
            <div class="folder-content" @click="showFolderContent(item)">
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

    <!-- 删除全部提醒模态框 -->
    <div v-if="showDeleteModal" class="modal">
      <div class="modal-content">
        <h3 v-if="settings.keep_favorites_on_delete">确定要清空所有未收藏的历史记录吗？此操作不可撤销！</h3>
        <h3 v-else>确定要清空所有历史记录吗？此操作不可撤销！</h3>
        <div class="modal-actions-center">
          <button @click="cancelDeleteAll" class="btn btn-secondary">取消</button>
          <button @click="deleteAllHistory" class="btn btn-least">删除</button>
        </div>
      </div>
    </div>

    <!-- 删除单条提醒模态框 -->
    <div v-if="showDeleteSingleModal" class="modal">
      <div class="modal-content">
        <h3>确定要删除这条历史记录吗？此操作不可撤销！</h3>
        <div class="modal-actions-center">
          <button @click="cancelDeleteSingle" class="btn btn-secondary">取消</button>
          <button @click="removeItem" class="btn btn-least">删除</button>
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
                <span class="content-count">{{ item.num_items }}个内容</span>
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
import { useClipboardApp } from '../composables/ClipboardApp'
import { 
  BeakerIcon,
  Cog6ToothIcon,
  ArrowPathIcon,
  LockClosedIcon,
  StarIcon,
  PencilSquareIcon,
  ClipboardDocumentListIcon,
  TrashIcon,
  Square2StackIcon,
  FolderPlusIcon,
  FolderIcon,
  LockOpenIcon
 } from '@heroicons/vue/24/outline'
import { 
  StarIcon as StarIconSolid,
  Square2StackIcon as Square2StackIconSolid
} from '@heroicons/vue/24/solid'

// 使用组合函数获取所有状态和方法
const {
  // 状态
  searchQuery,
  activeCategory,
  showToast,
  toastMessage,
  showEditModal,
  showNoteModal,
  showFolderModal,
  showFoldersModal,
  showOcrModal,
  showDeleteModal,
  showDeleteSingleModal,
  editingText,
  editingItem,
  notingText,
  notingItem,
  ocrText,
  folderNotingText,
  currentFolder,
  currentItem,
  folderQuery,
  searchLoading,
  canDeleteWindow,
  test,
  multiSelectMode,
  selectedItems,
  showMultiCopyBtn,
  startTime,
  endTime,
  categories,
  folders,
  filteredHistory,
  initialSelectedFolders,
  iconCache,

  // 计算属性
  selectedItemsCount,
  normalizedPath,
  settings,

  // 方法
  convertFileSrc,
  showMessage,
  setActiveCategory,
  togglePinnedView,
  openSettings,
  handleSearch,
  copyItem,
  toggleFavorite,
  executeDoubleClick,
  editItem,
  saveEdit,
  cancelEdit,
  noteItem,
  saveNote,
  cancelNote,
  showOCR,
  copyOCR,
  cancelOCR,
  removeItem,
  showFolder,
  addFolder,
  cancelFolder,
  removeFolder,
  addFolderToast,
  showFolderContent,
  selectFolder,
  addToFolder,
  cancelAddToFolder,
  showDeleteAll,
  showDeleteSingle,
  deleteAllHistory,
  cancelDeleteAll,
  deleteSingle,
  cancelDeleteSingle,
  handleItemClick,
  copySelectedItems,
  exitMultiSelectMode,
  getAllHistory,
  getAllFolders,
  formatTime,
  formatFileSize,
  getFileName,
  handleImageError,
  startDragging,
  setupWindowListeners,
  removeWindowListeners
} = useClipboardApp()
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
  gap: 3px;
}

.search-bar {
  display: flex;
  flex-direction: row;
  justify-content: space-between;
  align-items: center;
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
  flex: 1;
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

/* 新增搜索类型选择器样式 */
.search-type-selector {
  margin-left: 4px;
  flex-shrink: 0;
}

.search-type-select {
  padding: 6px 8px;
  border: 1px solid #e1e8ed;
  border-radius: 8px;
  font-size: 14px;
  outline: none;
  background: white;
  color: #333;
  cursor: pointer;
  transition: all 0.2s;
  min-width: 80px;
}

.search-type-select:hover {
  border-color: #b7c8fe;
}

.search-type-select:focus {
  border-color: #3282f6;
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.1);
}

/* 时间区间搜索输入框样式 */
.time-range-inputs {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.time-input-group {
  display: flex;
  align-items: center;
  gap: 6px;
  flex: 1;
}

.time-input-group label {
  font-size: 12px;
  color: #666;
  white-space: nowrap;
}

.time-input {
  border: none;
  border-radius: 8px;
  font-size: 20px;
  outline: none;
  transition: all 0.2s;
  color: transparent;
  position: relative;
  width: 24px;
}

/* 工具栏样式 */
.toolbar {
  display: flex;
  justify-content: space-between;
  padding: 8px 10px;
  background: #ffffff;
  align-items: center;
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
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

.icon-notes {
  width: 1rem;
  height: 1rem;
  position: relative;
  top: 3px; 
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
  user-select: none;
  -webkit-user-select: none;
  -moz-user-select: none;
  -ms-user-select: none;
}

.history-item:hover {
  border-color: #b7c8fe;
}

.history-item:focus {
  border-color: #3282f6;
  box-shadow: 0 0 0 2px rgba(52, 152, 219, 0.1);
  outline: none;
}

/* 多选模式样式 */
.history-item.selected {
  border-color: #3282f6;
  background-color: #e4edfd;
}

/* 信息框架 */
.item-info {
  display: flex;
  justify-content: space-between;
}

/* 多选序号 */
.selection-order-badge {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 20px;
  height: 20px;
  background: #3282f6;
  color: white;
  border-radius: 50%;
  font-size: 12px;
  font-weight: bold;
  margin-right: 8px;
  flex-shrink: 0;
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