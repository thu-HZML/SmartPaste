// src/utils/actions.js
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'

// 存储所有窗口实例
const windowInstances = new Map()

// 全局状态存储主窗口位置
let mainWindowPosition = { x: 100, y: 100, width: 200, height: 200 }

/**
 * 更新主窗口位置（在主窗口组件中调用）
 */
export function updateMainWindowPosition(position, size) {
  mainWindowPosition = {
    x: position.x,
    y: position.y,
    width: size.width,
    height: size.height
  }
  console.log('更新主窗口位置:', mainWindowPosition)
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
      url: '/menu',
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
      try {
        await window.close()
        windowInstances.delete(windowId)
      } catch (error) {
        console.error('关闭窗口失败:', error)
      }
    }
    return null
  } else {
    // 如果不存在，创建新窗口
    try {
      // 使用全局存储的主窗口位置
      const { x, y, width, height } = mainWindowPosition
      
      // 计算新窗口位置（在桌宠右侧）
      const newX = x + width + 10
      const newY = y
      
      console.log('使用主窗口位置创建剪贴板窗口:', { newX, newY })
      
      return await createClipboardWindow({
        x: newX,
        y: newY,
        width: 400,
        height: 600
      })
    } catch (error) {
      console.error('创建剪贴板窗口错误:', error)
      return await createClipboardWindow() // 创建默认位置的窗口
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
    .filter(([key]) => key.startsWith('clipboard_'))
  
  for (const [windowId, window] of clipboardWindows) {
    try {
      await window.close()
      windowInstances.delete(windowId)
    } catch (error) {
      console.error('关闭窗口失败:', error)
    }
  }
}

// 将函数暴露给全局，方便 Tauri 调用
if (typeof window !== 'undefined') {
  window.toggleClipboardWindow = toggleClipboardWindow;
}