// src/utils/actions.js
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { getCurrent } from '@tauri-apps/api/window'

// 存储所有窗口实例
const windowInstances = new Map()

/**
 * 创建登录窗口
 */
export async function loginWin() {
  try {
    const webview = new WebviewWindow('login', {
      url: '/login',
      title: '登录',
      width: 400,
      height: 500,
      resizable: false,
      center: true,
      decorations: false,
      alwaysOnTop: true
    })
    
    webview.once('tauri://created', () => {
      console.log('登录窗口创建成功')
      windowInstances.set('login', webview)
    })
    
    webview.once('tauri://error', (e) => {
      console.error('登录窗口创建失败:', e)
    })
    
    return webview
  } catch (error) {
    console.error('创建登录窗口错误:', error)
  }
}

/**
 * 创建剪贴板窗口
 * @param {Object} options 窗口配置
 */
export async function createClipboardWindow(options = {}) {
  const windowId = `clipboard_${Date.now()}`
  
  try {
    const { x = 100, y = 100, width = 400, height = 600 } = options
    
    const webview = new WebviewWindow(windowId, {
      url: '/clipboard',
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
    })
    
    webview.once('tauri://error', (e) => {
      console.error('剪贴板窗口创建失败:', e)
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
  // 查找已存在的剪贴板窗口
  const clipboardWindows = Array.from(windowInstances.entries())
    .filter(([key]) => key.startsWith('clipboard_'))
  
  if (clipboardWindows.length > 0) {
    // 如果存在剪贴板窗口，关闭它们
    for (const [windowId, window] of clipboardWindows) {
      await window.close()
      windowInstances.delete(windowId)
    }
    return null
  } else {
    // 如果不存在，创建新窗口
    const currentWindow = getCurrent()
    const position = await currentWindow.innerPosition()
    const size = await currentWindow.innerSize()
    
    // 计算新窗口位置（在桌宠右侧）
    const newX = position.x + size.width + 10
    const newY = position.y
    
    return await createClipboardWindow({
      x: newX,
      y: newY,
      width: 400,
      height: 600
    })
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
    await window.close()
    windowInstances.delete(windowId)
  }
}

/**
 * 关闭所有剪贴板窗口
 */
export async function closeAllClipboardWindows() {
  const clipboardWindows = Array.from(windowInstances.entries())
    .filter(([key]) => key.startsWith('clipboard_'))
  
  for (const [windowId, window] of clipboardWindows) {
    await window.close()
    windowInstances.delete(windowId)
  }
}