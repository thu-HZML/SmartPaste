// src/utils/actions.js
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { LogicalPosition, LogicalSize } from '@tauri-apps/api/window'
import { emit } from '@tauri-apps/api/event';
import { useSettingsStore } from '../stores/settings'; 
import { deleteAllData, deleteUnfavoritedData } from '../services/api';
import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';

// 存储所有窗口实例
export const windowInstances = new Map()

// 全局状态存储主窗口位置
let mainWindowPosition = { x: 100, y: 100 }

let aiAgentWindowHeight = 70

// 添加：获取屏幕工作区尺寸
let screenWorkArea = { x: 0, y: 0, width: 1920, height: 1080 }

/**
 * 更新主窗口位置
 */
export function updateMainWindowPosition(position) {
  mainWindowPosition = {
    x: position.x,
    y: position.y,
  }
}

/**
 * 更新屏幕工作区尺寸
 */
export async function updateScreenWorkArea() {
  try {
    /*
    const allWindows = await WebviewWindow.getAll()
    const mainWindow = allWindows.find(win => win.label === 'main')
    
    if (mainWindow) {
      const monitor = await mainWindow.currentMonitor()
      if (monitor && monitor.workArea) {
        screenWorkArea = {
          x: monitor.workArea.x,
          y: monitor.workArea.y,
          width: monitor.workArea.width,
          height: monitor.workArea.height
        }
        console.log('更新屏幕工作区:', screenWorkArea)
      }
    }
      */
  } catch (error) {
    console.error('获取屏幕工作区失败:', error)
  }
}

/**
 * 检查位置是否在屏幕工作区内
 * @param {Object} position 位置 {x, y}
 * @param {Object} size 窗口大小 {width, height}
 * @returns {boolean} 是否在屏幕内
 */
function isPositionInScreen(position, size) {
  const { x, y } = position
  const { width, height } = size
  
  // 检查窗口是否完全在屏幕工作区内
  return (
    x >= screenWorkArea.x &&
    y >= screenWorkArea.y &&
    x + width <= screenWorkArea.x + screenWorkArea.width &&
    y + height <= screenWorkArea.y + screenWorkArea.height
  )
}

/**
 * 计算Menu窗口的最佳位置
 * @param {number} menuWidth 菜单窗口宽度
 * @param {number} menuHeight 菜单窗口高度
 * @returns {Object} 最佳位置 {x, y}
 */
function calculateMenuWindowPosition(menuWidth, menuHeight) {
  // 尝试右下角位置（原方案）
  const rightBottomPosition = {
    x: mainWindowPosition.x + 150,
    y: mainWindowPosition.y
  }
  
  // 检查右下角位置是否有效
  if (isPositionInScreen(rightBottomPosition, { width: menuWidth, height: menuHeight })) {
    console.log('Menu窗口使用右下角位置')
    return rightBottomPosition
  }
  
  // 如果右下角位置无效，尝试左上角位置
  const leftTopPosition = {
    x: mainWindowPosition.x - menuWidth - 10, // 左上角，留10px间隙
    y: mainWindowPosition.y - menuHeight - 10
  }
  
  // 检查左上角位置是否有效
  if (isPositionInScreen(leftTopPosition, { width: menuWidth, height: menuHeight })) {
    console.log('Menu窗口使用左上角位置')
    return leftTopPosition
  }
  
  // 如果两个位置都无效，强制调整到屏幕内
  console.log('Menu窗口强制调整到屏幕内')
  return adjustPositionToScreen(leftTopPosition, { width: menuWidth, height: menuHeight })
}

/**
 * 调整位置确保在屏幕内
 * @param {Object} position 原始位置 {x, y}
 * @param {Object} size 窗口大小 {width, height}
 * @returns {Object} 调整后的位置 {x, y}
 */
function adjustPositionToScreen(position, size) {
  let { x, y } = position
  const { width, height } = size
  
  // 检查X轴
  if (x < screenWorkArea.x) {
    x = screenWorkArea.x
  } else if (x + width > screenWorkArea.x + screenWorkArea.width) {
    x = screenWorkArea.x + screenWorkArea.width - width
  }
  
  // 检查Y轴
  if (y < screenWorkArea.y) {
    y = screenWorkArea.y
  } else if (y + height > screenWorkArea.y + screenWorkArea.height) {
    y = screenWorkArea.y + screenWorkArea.height - height
  }
  
  return { x, y }
}

/**
 * 更新ai窗口高度
 */
export function updateAiWindowHeight(height) {
  aiAgentWindowHeight = height
  console.log('更新全局ai窗口高度:', aiAgentWindowHeight)
}

/**
 * 创建菜单窗口
 * @param {Object} options 窗口配置
 */
export async function createMenuWindow(options = {}) {
  const windowId = 'menu'
  
  try {
    const { x = 100, y = 100, width = 400, height = 600 } = options
    
    const webview = new WebviewWindow(windowId, {
      url: '/menu', // 使用你的菜单路由
      title: '主菜单',
      width,
      height,
      x,
      y,
      resizable: false, // 菜单通常不需要调整大小
      minimizable: true,
      maximizable: false,
      decorations: false, // 无边框
      alwaysOnTop: true,
      skipTaskbar: true,
      hiddenTitle: true,
      focus: true // 获取焦点
    })
    
    webview.once('tauri://created', () => {
      console.log('菜单窗口创建成功:', windowId)
      windowInstances.set(windowId, webview)
    })
    
    webview.once('tauri://error', (e) => {
      console.error('菜单窗口创建失败:', e)
    })
    
    // 监听窗口关闭
    webview.listen('tauri://destroyed', () => {
      console.log('菜单窗口已关闭:', windowId)
      windowInstances.delete(windowId)
    })
    
    return webview
  } catch (error) {
    console.error('创建菜单窗口错误:', error)
  }
}

/**
 * 获取或切换菜单窗口
 */
export async function toggleMenuWindow() {
  // 首先更新屏幕工作区信息
  await updateScreenWorkArea()
  
  // 查找已存在的菜单窗口
  const menuWindow = Array.from(windowInstances.entries())
    .find(([key]) => key === 'menu')
  
  if (menuWindow) {
    // 如果存在菜单窗口，关闭
    try {
      const [windowId, window] = menuWindow
      await window.close()
      windowInstances.delete(windowId)
    } catch (error) {
      console.error('关闭菜单窗口失败:', error)
    }
    return null
  } else {
    // 如果不存在，创建新窗口
    try {
      // 计算最佳位置
      const menuWidth = 300
      const menuHeight = 350
      const bestPosition = calculateMenuWindowPosition(menuWidth, menuHeight)
      
      console.log('Menu窗口位置计算:', {
        主窗口位置: mainWindowPosition,
        屏幕工作区: screenWorkArea,
        计算出的最佳位置: bestPosition
      })
      
      return await createMenuWindow({
        x: bestPosition.x,
        y: bestPosition.y,
        width: menuWidth,
        height: menuHeight
      })
    } catch (error) {
      console.error('创建菜单窗口错误:', error)
      return await createMenuWindow() // 创建默认位置的窗口
    }
  }
}

// 新增：更新菜单窗口位置函数
export async function updateMenuWindowPosition() {
  const menuWindow = Array.from(windowInstances.entries())
    .find(([key]) => key === 'menu')
  
  if (menuWindow) {
    // 更新屏幕工作区信息
    await updateScreenWorkArea()
    
    const menuWidth = 300
    const menuHeight = 350
    const bestPosition = calculateMenuWindowPosition(menuWidth, menuHeight)
    
    const [windowId, window] = menuWindow
    try {
      await window.setPosition(new LogicalPosition(bestPosition.x, bestPosition.y))
      console.log('更新Menu窗口位置:', bestPosition)
    } catch (error) {
      console.error('更新菜单窗口位置失败:', error)
    }
  }
}

/**
 * 检查是否有菜单窗口打开
 */
export function hasMenuWindow() {
  return Array.from(windowInstances.keys()).some(key => key.startsWith('menu'))
}

/**
 * 创建剪贴板窗口
 * @param {Object} options 窗口配置
 */
export async function createClipboardWindow(options = {}) {
  //const windowId = `clipboard_${Date.now()}`
  const windowId = 'clipboard'
  try {
    const { x = 100, y = 100, width = 400, height = 600 } = options
    
    const webview = new WebviewWindow(windowId, {
      url: '/clipboardapp',
      title: '剪贴板',
      width,
      height,
      x,
      y,
      resizable: true,
      minimizable: true,
      maximizable: false,
      decorations: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      hiddenTitle: true
    })
    
    webview.once('tauri://created', () => {
      console.log('剪贴板窗口创建成功:', windowId)
      windowInstances.set(windowId, webview)
      console.log(windowInstances)
    })
    
    webview.once('tauri://error', (e) => {
      const currentWindowId = 'clipboard';
      // 检查是否是已知的 'already exists' 竞态错误
      if (e.payload && typeof e.payload === 'string' && e.payload.includes('already exists')) {
        console.warn(`剪贴板窗口创建警告: 窗口 '${currentWindowId}' 正在清理中，无法立即创建。已忽略此错误。`);
      } else {
        // 其他错误，需要报告
        console.error('剪贴板窗口创建失败:', e)
      }
    })
    
    // 监听窗口关闭
    webview.listen('tauri://destroyed', () => {
      console.log('剪贴板窗口已关闭:', windowId)
      windowInstances.delete(windowId)
    })
    
    return webview
  } catch (error) {
    console.error('创建剪贴板窗口错误:', error)
  }
}

/**
 * 获取或切换剪贴板窗口
 */
export async function toggleClipboardWindow() {
  const windowId = 'clipboard'
  const allWindows = await WebviewWindow.getAll()
  const clipboardWindowInstance = allWindows.find(w => w.label === windowId)

  if (clipboardWindowInstance) {
    // 如果存在剪贴板窗口，关闭
    try {
      console.log('关闭剪贴板窗口 (通过 getAll 获得的完整实例)')
      await clipboardWindowInstance.close()
    } catch (error) {
      console.error('关闭剪贴板窗口失败:', error)
      return
    }
    return null
  } else {
    // 如果不存在，创建新窗口
    try {
      const savedState = localStorage.getItem('clipboardWindowState')
      
      if (savedState) {
        const windowState = JSON.parse(savedState)
        
        // 检查保存的状态是否在合理范围内（防止窗口出现在屏幕外）
        const { x, y, width, height } = windowState
        
        if (width > 0 && height > 0) {
          await createClipboardWindow({ // 创建上次位置的窗口
            x: x,
            y: y,
            width: width, // 菜单窗口宽度
            height: height // 菜单窗口高度
          })
        }
      } else {
        await createClipboardWindow() // 创建默认位置的窗口
      }
    } catch (error) {
      // 捕获并忽略 'already exists' 错误
      if (error.payload && typeof error.payload === 'string' && error.payload.includes('already exists')) {
          console.warn('⚠️ 窗口标签仍被占用（正在清理中），无法立即创建新窗口。')
      } else {
          console.error('创建剪贴板窗口错误:', error)
      }
    }
  }
}

/**
 * 创建收藏夹窗口
 * @param {Object} options 窗口配置
 */
export async function createFavoritesWindow(options = {}) {
  const windowId = 'clipboard'
  
  try {
    const { x = 100, y = 100, width = 400, height = 600 } = options
    
    const webview = new WebviewWindow(windowId, {
      url: '/clipboardapp?category=favorite', // 直接跳转到剪贴板页面的收藏界面
      title: '收藏夹',
      width,
      height,
      x,
      y,
      resizable: true,
      minimizable: true,
      maximizable: false,
      decorations: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      hiddenTitle: true,
      focus: true
    })
    
    webview.once('tauri://created', () => {
      console.log('收藏夹窗口创建成功:', windowId)
      windowInstances.set(windowId, webview)
    })
    
    webview.once('tauri://error', (e) => {
      console.error('收藏夹窗口创建失败:', e)
    })
    
    // 监听窗口关闭
    webview.listen('tauri://destroyed', () => {
      console.log('收藏夹窗口已关闭:', windowId)
      windowInstances.delete(windowId)
    })
    
    return webview
  } catch (error) {
    console.error('创建收藏夹窗口错误:', error)
  }
}

/**
 * 获取或切换收藏夹窗口
 */
export async function toggleFavoritesWindow() {
  // 查找已存在的收藏夹窗口
  const favoritesWindow = Array.from(windowInstances.entries())
    .find(([key]) => key === 'clipboard')
  
  if (favoritesWindow) {
    // 如果存在收藏夹窗口，关闭
    try {
      const [windowId, window] = favoritesWindow
      await window.close()
      windowInstances.delete(windowId)
    } catch (error) {
      console.error('关闭收藏夹窗口失败:', error)
    }
    return null
  } else {
    // 如果不存在，创建新窗口
    try {
      const savedState = localStorage.getItem('clipboardWindowState')
      
      if (savedState) {
        const windowState = JSON.parse(savedState)
        
        // 检查保存的状态是否在合理范围内（防止窗口出现在屏幕外）
        const { x, y, width, height } = windowState
        
        if (width > 0 && height > 0) {
          await createFavoritesWindow({ // 创建上次位置的窗口
            x: x,
            y: y,
            width: width, // 菜单窗口宽度
            height: height // 菜单窗口高度
          })
        }
      } else {
        await createFavoritesWindow() // 创建默认位置的窗口
      }
    } catch (error) {
      console.error('创建收藏夹窗口错误:', error)
      return await createFavoritesWindow() // 创建默认位置的窗口
    }
  }
}

/**
 * 创建隐私窗口
 * @param {Object} options 窗口配置
 */
export async function createPrivateWindow(options = {}) {
  const windowId = 'clipboard'
  
  try {
    const { x = 100, y = 100, width = 400, height = 600 } = options
    
    const webview = new WebviewWindow(windowId, {
      url: '/clipboardapp?category=private', // 直接跳转到剪贴板页面的隐私界面
      title: '隐私',
      width,
      height,
      x,
      y,
      resizable: true,
      minimizable: true,
      maximizable: false,
      decorations: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      hiddenTitle: true,
      focus: true
    })
    
    webview.once('tauri://created', () => {
      console.log('隐私窗口创建成功:', windowId)
      windowInstances.set(windowId, webview)
    })
    
    webview.once('tauri://error', (e) => {
      console.error('隐私窗口创建失败:', e)
    })
    
    // 监听窗口关闭
    webview.listen('tauri://destroyed', () => {
      console.log('隐私窗口已关闭:', windowId)
      windowInstances.delete(windowId)
    })
    
    return webview
  } catch (error) {
    console.error('创建隐私窗口错误:', error)
  }
}

/**
 * 获取或切换隐私窗口
 */
export async function togglePrivateWindow() {
  // 查找已存在的隐私窗口
  const privateWindow = Array.from(windowInstances.entries())
    .find(([key]) => key === 'clipboard')
  
  if (privateWindow) {
    // 如果存在收藏夹窗口，关闭
    try {
      const [windowId, window] = privateWindow
      await window.close()
      windowInstances.delete(windowId)
    } catch (error) {
      console.error('关闭隐私窗口失败:', error)
    }
    return null
  } else {
    // 如果不存在，创建新窗口
    try {
      const savedState = localStorage.getItem('clipboardWindowState')
      
      if (savedState) {
        const windowState = JSON.parse(savedState)
        
        // 检查保存的状态是否在合理范围内（防止窗口出现在屏幕外）
        const { x, y, width, height } = windowState
        
        if (width > 0 && height > 0) {
          await createPrivateWindow({ // 创建上次位置的窗口
            x: x,
            y: y,
            width: width, // 剪贴板窗口宽度
            height: height // 剪贴板窗口高度
          })
        }
      } else {
        await createPrivateWindow() // 创建默认位置的窗口
      }
    } catch (error) {
      console.error('创建隐私窗口错误:', error)
      return await createPrivateWindow() // 创建默认位置的窗口
    }
  }
}

// 创建设置窗口
export async function createSetWindow(options = {},initialNav = 'general') {
  const windowId = 'preferences'
  
  try {
    const { x = 100, y = 100, width = 800, height = 580 } = options
    
    const webview = new WebviewWindow(windowId, {
      url: `/preferences?nav=${initialNav}`,
      title: '设置',
      width,
      height,
      x,
      y,
      resizable: true,
      minimizable: false,
      maximizable: false,
      decorations: true,
      alwaysOnTop: true,
      skipTaskbar: true,
      hiddenTitle: true,
      focus: true
    })
    
    webview.once('tauri://created', () => {
      console.log('设置窗口创建成功:', windowId)
      windowInstances.set(windowId, webview)
    })
    
    webview.once('tauri://error', (e) => {
      // 检查是否是已知的 'already exists' 竞态错误
      if (e.payload && typeof e.payload === 'string' && e.payload.includes('already exists')) {
        // 忽略此错误，因为它是异步清理未完成时尝试重新创建导致的常见错误
        console.warn(`设置窗口创建警告: 窗口 '${windowId}' 正在清理中，无法立即创建。已忽略此错误。`);
      } else {
        console.error('设置窗口创建失败:', e);
      }
    })
    
    // 监听窗口关闭
    webview.listen('tauri://destroyed', () => {
      console.log('设置窗口已关闭:', windowId)
      windowInstances.delete(windowId)
    })
    
    return webview
  } catch (error) {
    console.error('创建设置窗口错误:', error)
  }
}

/**
 * 获取或切换设置窗口
 */
export async function toggleSetWindow(navId = 'general') {
  const windowId = 'preferences'
  const allWindows = await WebviewWindow.getAll()
  const setsWindowInstance = allWindows.find(w => w.label === windowId)
  
  if (setsWindowInstance) {
    // 如果存在设置窗口，关闭
    try {
      console.log('关闭设置窗口 (全局查找)')
      await setsWindowInstance.close()

    } catch (error) {
      console.error('关闭设置窗口失败:', error)
      return
    }
  } else {
    // 如果不存在，创建新窗口
    try {
      const savedState = localStorage.getItem('preferencesWindowState')
      console.log('设置窗口全局位置：', savedState)
      if (savedState) {
        const windowState = JSON.parse(savedState)
        
        // 检查保存的状态是否在合理范围内（防止窗口出现在屏幕外）
        const { x, y, width, height } = windowState
        
        if (width > 0 && height > 0) {
          await createSetWindow({ // 创建上次位置的窗口
            x: x,
            y: y,
            width: width, // 菜单窗口宽度
            height: height // 菜单窗口高度
          },navId)
        }
      } else {
        await createSetWindow(navId) // 创建默认位置的窗口
      }
    } catch (error) {
      if (error.payload && typeof error.payload === 'string' && error.payload.includes('already exists')) {
          console.warn('⚠️ 窗口标签仍被占用（正在清理中），无法立即创建新窗口。')
      } else {
          console.error('创建设置窗口失败:', error)
      }
    }
  }
}

// 创建ai窗口
export async function createAiWindow(options = {}) {
  const windowId = 'aiAgent'
  
  try {
    const { x = 100, y = 100, width = 800, height = 580 } = options
    
    const webview = new WebviewWindow(windowId, {
      url: '/aiagent',
      title: 'ai助手',
      width,
      height,
      x,
      y,
      resizable: true,
      minimizable: false,
      maximizable: false,
      decorations: false,
      alwaysOnTop: true,
      skipTaskbar: true,
      hiddenTitle: true,
      focus: false,
      transparent: true,
      shadow: false,
    })
    
    webview.once('tauri://created', () => {
      console.log('ai窗口创建成功:', windowId)
      windowInstances.set(windowId, webview)
    })
    
    webview.once('tauri://error', (e) => {
      // 检查是否是已知的 'already exists' 竞态错误
      if (e.payload && typeof e.payload === 'string' && e.payload.includes('already exists')) {
        // 忽略此错误，因为它是异步清理未完成时尝试重新创建导致的常见错误
        console.warn(`ai窗口创建警告: 窗口 '${windowId}' 正在清理中，无法立即创建。已忽略此错误。`);
      } else {
        console.error('ai窗口创建失败:', e);
      }
    })
    
    // 监听窗口关闭
    webview.listen('tauri://destroyed', () => {
      console.log('ai窗口已关闭:', windowId)
      windowInstances.delete(windowId)
    })
    
    return webview
  } catch (error) {
    console.error('创建ai窗口错误:', error)
  }
}

/**
 * 获取或切换ai窗口
 */
export async function toggleAiWindow() {
  const windowId = 'aiAgent'
  const allWindows = await WebviewWindow.getAll()
  const aiWindowInstance = allWindows.find(w => w.label === windowId)
  
  if (aiWindowInstance) {
    // 如果存在ai窗口，关闭
    return
  } else {
    // 如果不存在，创建新窗口
    try {
      // 使用全局存储的主窗口位置
      const { x, y } = mainWindowPosition
      
      // 计算新窗口位置（在桌宠上方）
      const newX = x - 250
      const newY = y - aiAgentWindowHeight
      
      console.log('使用主窗口位置创建ai窗口:', { 
        mainWindow: { x, y },
        menuWindow: { newX, newY }
      })
      
      return await createAiWindow({
        x: newX,
        y: newY,
        width: 400, // 菜单窗口宽度
        height: 80 // 菜单窗口高度
      })
    } catch (error) {
      console.error('使用主窗口位置创建ai窗口错误:', error)
      return await createAiWindow() // 创建默认位置的窗口
    }
  }
}

// 更新ai窗口位置函数
export async function updateAiWindowPosition() {
  const savedState = localStorage.getItem('aiWindowHeight')
  if (savedState) {
    const windowState = JSON.parse(savedState)        
    aiAgentWindowHeight = windowState.height
  }

  const aiWindow = Array.from(windowInstances.entries())
    .find(([key]) => key === 'aiAgent')
  
  if (aiWindow) {
    const { x, y } = mainWindowPosition
    const newX = x - 250
    const newY = y - aiAgentWindowHeight

    const [windowId, window] = aiWindow
    try {
      await window.setPosition(new LogicalPosition(newX, newY))
    } catch (error) {
      console.error('更新ai窗口位置失败:', error)
    }
  }
}

/**
 * 获取所有窗口信息
 */
export function getAllWindows() {
  return Array.from(windowInstances.entries()).map(([id, window]) => ({
    id,
    window
  }))
}

/**
 * 通过ID关闭窗口
 */
export async function closeWindowById(windowId) {
  const window = windowInstances.get(windowId)
  if (window) {
    try {
      await window.close()
      windowInstances.delete(windowId)
    } catch (error) {
      console.error('关闭窗口失败:', error)
    }
  }
}

/**
 * 关闭所有剪贴板窗口
 */
export async function closeAllClipboardWindows() {
  const clipboardWindows = Array.from(windowInstances.entries())
    .filter(([key]) => key.startsWith('clipboard'))
  
  for (const [windowId, window] of clipboardWindows) {
    try {
      await window.close()
      windowInstances.delete(windowId)
    } catch (error) {
      console.error('关闭窗口失败:', error)
    }
  }
}

/**
 * 关闭所有菜单窗口
 */
export async function closeAllMenuWindows() {
  const menuWindows = Array.from(windowInstances.entries())
    .filter(([key]) => key.startsWith(''))
  
  for (const [windowId, window] of menuWindows) {
    try {
      await window.close()
      windowInstances.delete(windowId)
    } catch (error) {
      console.error('关闭菜单窗口失败:', error)
    }
  }
}

export async function clearClipboardHistory() {
    console.log('JS: clearClipboardHistory called by shortcut');

    try {
        const settingsStore = useSettingsStore();
        await settingsStore.initializeSettings();
        const settings = settingsStore.settings; 
        
        let confirmed = true;
        
        // 删除确认对话框
        if (settings.delete_confirmation) {
            const message = settings.keep_favorites_on_delete
                ? '确定要清空所有未收藏的剪贴板历史吗？'
                : '确定要清空所有历史记录吗？';            
            confirmed = await window.confirm(message);;
            console.log(`[DEBUG] window.confirm 返回值 (confirmed): ${confirmed}`);
        }
        if (!confirmed) {
            console.log(`[DEBUG] window.confirm 返回值 (confirmed): ${confirmed}`);
            return;
        }
        let messageText = '';
        let rowsAffected = 0;

        // 执行删除操作
        if (settings.keep_favorites_on_delete) {
            rowsAffected = await deleteUnfavoritedData();
            messageText = '已清除所有未收藏记录';
        } else {
            rowsAffected = await deleteAllData();
            messageText = '已清除所有历史记录';
        }

        console.log(`快捷键清空操作完成: ${messageText}，共 ${rowsAffected} 条记录被删除`);


        //  **发送 Tauri 系统通知 (实现全局反馈)**
        let permissionGranted = await isPermissionGranted();
        console.log('通知权限状态 (初始):', permissionGranted); // 检查初始权限

        if (!permissionGranted) {
            const permission = await requestPermission();
            permissionGranted = permission === 'granted';
            console.log('通知权限状态 (请求后):', permissionGranted); // 检查请求后的权限
        }

        if (permissionGranted) {
            console.log('正在发送通知...'); // 确认 sendNotification 即将执行
            sendNotification({
                title: '剪贴板历史清理',
                body: `${messageText}。共删除 ${rowsAffected} 条记录。`
            });
        }

        // 4. 通知前端主组件进行 UI 刷新 (如果 ClipboardApp.vue 正在运行，它将刷新)
        await emit('clipboard-history-cleared', { 
            message: messageText, 
            rows: rowsAffected
        }); 

    } catch (error) {
        console.error('清空剪贴板历史失败:', error);
        sendNotification({
            title: '剪贴板历史清理失败',
            body: `操作失败: ${error.message || error}`
        });
    }
}

// 将函数暴露给全局，方便 Tauri 调用
if (typeof window !== 'undefined') {
  window.toggleClipboardWindow = toggleClipboardWindow;
  window.toggleMenuWindow = toggleMenuWindow;
  window.toggleFavoritesWindow = toggleFavoritesWindow;
  window.toggleSetWindow = toggleSetWindow;
  window.updateMenuWindowPosition = updateMenuWindowPosition;
  window.hasMenuWindow = hasMenuWindow;
  window.clearClipboardHistory = clearClipboardHistory;
  window.mainWindowPosition = mainWindowPosition;
  window.updateScreenWorkArea = updateScreenWorkArea; // 添加这个函数
}