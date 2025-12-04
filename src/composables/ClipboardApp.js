// src/composables/useClipboardApp.js
import { ref, computed, onMounted, onUnmounted, watch, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
import { getCurrentWindow, LogicalSize, LogicalPosition } from '@tauri-apps/api/window'
import { toggleSetWindow } from '../utils/actions.js'
import { useSettingsStore } from '../stores/settings'

export function useClipboardApp() {
  const router = useRouter()
  const route = useRoute()
  const currentWindow = getCurrentWindow();
  const settings = useSettingsStore().settings        

  // 响应式数据
  const searchQuery = ref('')
  const activeCategory = ref('all')

  // 提示框&模态框
  const showToast = ref(false)
  const toastMessage = ref('')
  const showEditModal = ref(false)
  const showNoteModal = ref(false)
  const showFolderModal = ref(false)
  const showFoldersModal = ref(false)
  const showOcrModal = ref(false)
  const showDeleteModal = ref(false)
  const showDeleteSingleModal = ref(false)
  const editingText = ref('')
  const editingItem = ref(null)
  const notingText = ref('')
  const notingItem = ref(null)
  const ocrText = ref('')
  const folderNotingText = ref('')

  // 指向性数据
  const currentFolder = ref(null)
  const currentItem = ref(null)
  const folderQuery = ref('')
  const unlistenFocusChanged = ref(null) // 存储取消监听的函数

  // 状态显示
  const searchLoading = ref(false)
  const canDeleteWindow = ref(true)
  let isDragging = null
  const test = ref('')

  // 多选相关变量
  const multiSelectMode = ref(false)
  const selectedItems = ref([]) // 存储选中的历史记录
  const showMultiCopyBtn = ref(false) // 控制复制按钮显示

  // 搜索相关变量
  const searchType = ref('text') // 默认纯文本搜索
  const startTime = ref('')
  const endTime = ref('')

  // 防抖定时器
  let searchTimeout = null

  // 双击定时器
  let clickTimeout = null

  // 分类选项
  const categories = ref([
    { id: 'all', name: '全部' },
    { id: 'text', name: '文本' },
    { id: 'image', name: '图片' },
    { id: 'file', name: '文件' },
    { id: 'favorite', name: '收藏' }
  ])

  // 历史记录数据结构
  const folders = ref([])
  const filteredHistory = ref([])
  const initialSelectedFolders = ref([]) // 存储当前记录被收藏进的收藏夹
  const iconCache = ref({}) // 用于缓存已加载的图标

  // 计算属性
  const selectedItemsCount = computed(() => selectedItems.value.length)

  // 计算属性：根据搜索类型动态改变placeholder
  const searchPlaceholder = computed(() => {
    switch (searchType.value) {
      case 'text':
        return '搜索剪贴板内容...'
      case 'ocr':
        return '搜索图片OCR文字内容...'
      case 'path':
        return '搜索文件路径...'
      case 'time':
        return '请选择时间区间...'
      default:
        return '搜索剪贴板内容...'
    }
  })

  // 计算属性：规范化路径
  const normalizedPath = computed(() => {
    if (!settings.storage_path) return '未设置路径'
    return settings.storage_path.replace(/\//g, '\\') + '\\'
  })

  // 监听搜索类型变化
  watch(searchType, (newType) => {
    // 切换搜索类型时清空搜索框
    searchQuery.value = ''
    // 如果是时间搜索，初始化时间
    if (newType === 'time') {
      const now = new Date()
      const oneWeekAgo = new Date(now.getTime() - 7 * 24 * 60 * 60 * 1000)
      startTime.value = formatDateTimeLocal(oneWeekAgo)
      endTime.value = formatDateTimeLocal(now)
    } else {
      startTime.value = ''
      endTime.value = ''
    }
  })

  // 监听时间变化
  watch([startTime, endTime], async() => {
    if (searchType.value === 'time' && startTime.value && endTime.value) {
      await handleSearch('')
    }
  })

  // 监听 searchQuery 变化
  watch(searchQuery, async (newQuery) => {
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
    
    // 如果是收藏夹模式，不执行全局搜索(奇怪的bug: 收藏夹操作时可能会触发这里的搜索，导致内容被覆盖)
    if (activeCategory.value === 'folder') {
      console.log('收藏夹模式下跳过全局搜索')
      return
    }

    // 空查询立即返回
    if (query.trim() === '' && searchType.value !== 'time') {
      await getAllHistory()
      searchLoading.value = false
      return
    }
    
    searchLoading.value = true
    
    // 设置新的定时器（100ms 防抖）
    searchTimeout = setTimeout(async () => {
      await performSearch(query)
    }, 100)
  }

  // 分类逻辑
  const handleCategoryChange = async (category) => {
    if (['image', 'text', 'file'].includes(category)) {
      searchLoading.value = true
      await performClassify(category)
    }
    else if (category.trim() === 'all'){
      searchLoading.value = true
      await getAllHistory()
    }
    else if (category.trim() === 'favorite'){
      searchLoading.value = true
      await getAllFolders()
      const result = await invoke('get_favorite_data_count')
      folders.value[0].num_items = result
    }
    else if (category.trim() === 'folder') {
      console.log('当前收藏夹：', currentFolder.value.name)
      await performFolder()
    }

    if (multiSelectMode.value) {
      // 退出多选状态
      exitMultiSelectMode()
    }
  }

  // 搜索过滤
  const performSearch = async (query) => {
    try {
      let result = ''
      
      switch (searchType.value) {
        case 'text':
          result = await invoke('search_data', { 
            searchType: 'text',
            query: query.trim() 
          })
          break
        case 'ocr':
          result = await invoke('search_data_by_ocr_text', { 
            query: query.trim() 
          })
          break
        case 'path':
          result = await invoke('search_data', { 
            searchType: 'path',
            query: query.trim() 
          })
          break
        case 'time':
          if (startTime.value && endTime.value) {
            const startTimestamp = new Date(startTime.value).getTime()
            const endTimestamp = new Date(endTime.value).getTime()
            const timeRangeQuery = `${startTimestamp},${endTimestamp}`
            result = await invoke('search_data', { 
              searchType: 'timestamp',
              query: timeRangeQuery
            })
          } else {
            result = '[]'
          }
          break
        default:
          result = await invoke('search_text_content', { 
            query: query.trim() 
          })
      }
      
      filteredHistory.value = JSON.parse(result)

      // 为数组添加前端额外字段
      await optimizeHistoryItems(filteredHistory)
    } catch (err) {
      console.error('搜索失败:', err)
      showMessage('搜索失败: ' + err)
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

      // 为数组添加前端额外字段
      await optimizeHistoryItems(filteredHistory)
    } catch (err) {
      console.error('分类失败:', err)
    } finally {
      searchLoading.value = false
    }
  }

  // 收藏夹过滤
  const performFolder = async () => { 
    if (currentFolder.value.name === '全部') {
      console.log('进入全部收藏夹')
      try {
        const result = await invoke('filter_data_by_favorite', { 
          isFavorite: true
        })    
        filteredHistory.value = JSON.parse(result)

        console.log('全部收藏夹内容:', filteredHistory.value) 
        // 为数组添加前端额外字段
        await optimizeHistoryItems(filteredHistory)
        console.log('添加字段后全部收藏夹内容:', filteredHistory.value)
      } catch (err) {
        console.error('全部收藏夹获取失败:', err)
      } finally {
      searchLoading.value = false
    }
    }
    else {
      console.log('进入收藏夹：', currentFolder.value.name)
      try {
        const result = await invoke('filter_data_by_folder', { 
          folderName: currentFolder.value.name
        })    
        filteredHistory.value = JSON.parse(result)

        // 为数组添加前端额外字段
        await optimizeHistoryItems(filteredHistory)
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
    canDeleteWindow.value = !canDeleteWindow.value
    showMessage('切换固定视图')
  }

  // 打开设置
  const openSettings = async () => {
    // 移除窗口焦点监听器
    removeWindowListeners()
    await toggleSetWindow()
    currentWindow.close()
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
        const filePath = normalizedPath.value + item.content
        await invoke('write_file_to_clipboard', { filePath: filePath })
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
        
        // 如果项目已被收藏但不在任何收藏夹中，确保全部收藏夹被选中
        if (item.is_favorite) {
          const defaultFolder = folders.value.find(folder => folder.name === '全部')
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
      if (settings.keep_favorites_on_delete) {
        await invoke('delete_unfavorited_data')
        showMessage('已清除所有未收藏记录')
      } else {
        await invoke('delete_all_data')
        showMessage('已清除所有历史记录')
      }
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

  // 弹出"确认删除"提示框
  const showDeleteSingle = (item) => {
    currentItem.value = item
    if (settings.delete_confirmation) {
      showDeleteSingleModal.value = true
    } else {
      removeItem()
    }    
  }

  // "确认删除"提示框消失
  const cancelDeleteSingle = () => {
    showDeleteSingleModal.value = false
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
    const ocrString = await invoke('get_ocr_text_by_item_id', { itemId: item.id })
    console.log(ocrString)
    ocrText.value = ocrString
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
  const removeItem = async () => {
    const item = currentItem.value
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
      cancelDeleteSingle()
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

      // 为数组添加前端额外字段
      await optimizeHistoryItems(filteredHistory)
    } catch (error) {
      console.error('调用失败:', error)
    }
  }

  // 获取所有收藏夹
  const getAllFolders = async () => {
    try {
      const jsonString = await invoke('get_all_folders')
      folders.value = JSON.parse(jsonString)

      // 创建全部收藏夹
      if (!folders.value || folders.value.length === 0) {
        console.log('收藏夹为空，正在创建全部收藏夹...')
        try {
          await invoke('create_new_folder', { name: '全部' })
          console.log('全部收藏夹创建成功')
          
          // 重新获取收藏夹列表
          const updatedJsonString = await invoke('get_all_folders')
          folders.value = JSON.parse(updatedJsonString)
          console.log('更新后的收藏夹:', folders.value)
        } catch (createError) {
          console.error('创建全部收藏夹失败:', createError)
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
    // 如果是全部收藏夹
    if (item.name === '全部') {
      // 如果取消选中全部收藏夹，则取消所有其他收藏夹
      if (item.isSelected) {
        // 取消选中全部收藏夹
        item.isSelected = false
        // 取消所有其他收藏夹的选中
        folders.value.forEach(folder => {
          if (folder.name !== '全部') {
            folder.isSelected = false
          }
        })
      } else {
        // 选中全部收藏夹
        item.isSelected = true
      }
    } else {
      // 非全部收藏夹
      // 切换当前项的选中状态
      item.isSelected = !item.isSelected
      
      // 如果选中了任何非全部收藏夹，确保全部收藏夹也被选中
      if (item.isSelected) {
        const defaultFolder = folders.value.find(folder => folder.name === '全部')
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
        item.isSelected && item.name !== '全部'
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

      // 刷新历史记录
      handleSearch(searchQuery.value)
      handleCategoryChange(activeCategory.value)
    })
    
    return unlisten
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

    // 防止在选择框上触发拖动
    if (event.target.tagName === 'SELECT' || event.target.closest('select')) {
      return
    }
    
    // 防止在模态框上触发拖动
    if (event.target.closest('.modal')) {
      return
    }

    // 新增：防止在收藏夹项目上触发拖动
    if (event.target.closest('.folder-item')) {
      return
    }
    
    // 新增：防止在收藏夹内容区域上触发拖动
    if (event.target.closest('.folder-content')) {
      return
    }

    // 新增：防止在历史记录上触发拖动
    if (event.target.closest('.history-item')) {
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
        if (isDragging || !canDeleteWindow.value) {
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

  // 更新序号
  const updateSelectionOrder = () => {
    // 按选中顺序重新排序并分配序号
    selectedItems.value.forEach((item, index) => {
      item.selectionOrder = index + 1
    })
  }

  // 处理项目点击（支持Shift多选）
  const handleItemClick = (item, event) => {
    if (event.shiftKey && multiSelectMode.value) {
      // Shift多选逻辑
      const existingIndex = selectedItems.value.findIndex(selected => selected.id === item.id)
      
      if (existingIndex !== -1) {
        // 如果已经选中，则移除
        item.is_selected = false
        selectedItems.value.splice(existingIndex, 1)
        item.selectionOrder = 0 // 清除序号
      } else {
        // 如果未选中，则添加
        item.is_selected = true
        selectedItems.value.push(item)
      }
      
      // 更新复制按钮显示状态
      showMultiCopyBtn.value = selectedItems.value.length > 0

      // 更新所有选中项的序号
      updateSelectionOrder()

      event.stopPropagation()
    } else if (event.shiftKey && !multiSelectMode.value) {
      // 第一次按Shift点击，进入多选模式
      multiSelectMode.value = true
      item.is_selected = true
      selectedItems.value.push(item)
      showMultiCopyBtn.value = true
      showMessage('已进入多选模式，继续按Shift点击可选择多个项目')

      item.selectionOrder = 1 // 第一个项目序号为1

      event.stopPropagation()
    } else {
      exitMultiSelectMode()
    }
  }

  // 复制所有选中的项目
  const copySelectedItems = async () => {
    if (selectedItems.value.length === 0) {
      showMessage('请先选择要复制的项目')
      return
    }

    try {
      let successCount = 0
      let errorCount = 0
      let copyString = ''

      let filePaths = []

      selectedItems.value.forEach(item => {
        if (item.item_type === 'text') {
          copyString += item.content + '\n'
          successCount++
        }     
      })

      if (copyString.trim() !== '') {
        await invoke('write_to_clipboard', { text: copyString })
        showMessage(`已成功复制 ${successCount} 个文本项目`)
      } else {
        // 多选文件
        selectedItems.value.forEach(item => {
          if (item.item_type === 'file' || item.item_type === 'image' || item.item_type === 'folder') {
            filePaths.push(normalizedPath.value + item.content)
            successCount++
          }     
        })

        if (filePaths.length > 0) {
          console.log('准备复制的文件路径:', filePaths)
          await invoke('write_files_to_clipboard', { filePaths: filePaths })
          showMessage(`已成功复制 ${successCount} 个文件项目`)
        } else {
          showMessage('没有找到可复制的内容')
        }
      }

      // 复制完成后退出多选模式
      exitMultiSelectMode()
      
    } catch (error) {
      console.error('复制选中项目失败:', error)
      showMessage('复制失败，请重试')
    }
  }

  // 退出多选模式
  const exitMultiSelectMode = () => {
    multiSelectMode.value = false
    console.log('多选复制内容为：',selectedItems)
    selectedItems.value.forEach(item => {
      item.is_selected = false    
      item.selectionOrder = 0
    })
    selectedItems.value = []
    showMultiCopyBtn.value = false
  }

  // 为历史记录数组添加前端额外字段并获取图标数据
  async function optimizeHistoryItems(historyRef, options = {}) {
    const { defaultFocus = false, defaultSelected = false } = options
    const array = historyRef.value
    // 批量处理基础字段
    for (let i = 0; i < array.length; i++) {
      const item = array[i]
      item.is_focus = defaultFocus
      item.is_selected = defaultSelected
    }
    // 并行获取图标数据（带重试功能）
    const fileItems = array.filter(item => item.item_type === 'file' || item.item_type === 'folder')
    const promises = fileItems.map(item => 
      fetchIconWithRetryRecursive(item.id, 5) // 最多重试5次
        .then(iconString => {
          item.iconData = iconString
        })
        .catch(error => {
          console.error(`Failed to get icon for ${item.id} after retries:`, error)
          item.iconData = null
        })
    )   
    await Promise.all(promises)
  }

  // 递归版本的带重试功能的图标获取函数
  async function fetchIconWithRetryRecursive(itemId, retriesLeft = 5) {
    try {
      const iconString = await invoke('get_icon_data_by_item_id', { itemId })
      
      // 如果获取到的图标数据不为空，直接返回
      if (iconString && iconString.trim() !== '') {
        return iconString
      }
      // 如果为空且还有重试次数，等待100ms后递归调用
      if (retriesLeft > 0) {
        await new Promise(resolve => setTimeout(resolve, 100))
        return fetchIconWithRetryRecursive(itemId, retriesLeft - 1)
      } else {
        // 达到最大重试次数，返回空字符串
        console.warn(`Icon data for ${itemId} is empty after 5 retries.`)
        return ''
      }
    } catch (error) {
      // 如果发生错误且还有重试次数，等待100ms后递归调用
      if (retriesLeft > 0) {
        console.warn(`Failed to get icon for ${itemId}, retrying... (${5 - retriesLeft + 1}/5)`)
        await new Promise(resolve => setTimeout(resolve, 100))
        return fetchIconWithRetryRecursive(itemId, retriesLeft - 1)
      } else {
        // 达到最大重试次数，抛出错误
        throw new Error(`Failed to get icon after 5 retries: ${error}`)
      }
    }
  }

  // 辅助函数：格式化日期时间为datetime-local输入格式
  const formatDateTimeLocal = (date) => {
    const year = date.getFullYear()
    const month = String(date.getMonth() + 1).padStart(2, '0')
    const day = String(date.getDate()).padStart(2, '0')
    const hours = String(date.getHours()).padStart(2, '0')
    const minutes = String(date.getMinutes()).padStart(2, '0')
    
    return `${year}-${month}-${day}T${hours}:${minutes}`
  }

  // 生命周期
  onMounted(async () => {

    console.log('开始初始化...')

    // 保存当前参数状态
    const shouldShowFavorites = route.query.category === 'favorite'

    // 立即清除所有参数
    if (route.query.category) {
      const newQuery = { ...route.query }
      delete newQuery.category
      await router.replace({ query: newQuery })
      console.log('路由参数已清除')
    }

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

    // 根据保存的状态执行操作
    if (shouldShowFavorites) {
      activeCategory.value = 'favorite'
      console.log('跳转到收藏界面')
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

  onUnmounted(() => {
    removeWindowListeners()
  })

  return {
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
    searchType,
    startTime,
    endTime,
    categories,
    folders,
    filteredHistory,
    initialSelectedFolders,
    iconCache,

    // 计算属性
    selectedItemsCount,
    searchPlaceholder,
    normalizedPath,
    settings,

    // 方法
    convertFileSrc,
    showMessage,
    setActiveCategory,
    togglePinnedView,
    openSettings,
    handleSearch,
    handleCategoryChange,
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
  }
}