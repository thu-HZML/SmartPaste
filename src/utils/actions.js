// src/utils/actions.js
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { LogicalPosition } from '@tauri-apps/api/window'

// 存储所有窗口实例
export const windowInstances = new Map()

// 全局状态存储主窗口位置
let mainWindowPosition = { x: 100, y: 100, width: 200, height: 200 }

/**
 * 更新主窗口位置
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
 * 创建菜单窗口
 * @param {Object} options 窗口配置
 */
export async function createMenuWindow(options = {}) {
  const windowId = `menu_${Date.now()}`
  
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
  // 查找已存在的菜单窗口
  const menuWindows = Array.from(windowInstances.entries())
    .filter(([key]) => key.startsWith('menu_'))
  
  if (menuWindows.length > 0) {
    // 如果存在菜单窗口，关闭它们
    for (const [windowId, window] of menuWindows) {
      try {
        await window.close()
        windowInstances.delete(windowId)
      } catch (error) {
        console.error('关闭菜单窗口失败:', error)
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
      
      console.log('使用主窗口位置创建菜单窗口:', { 
        mainWindow: { x, y, width, height },
        menuWindow: { newX, newY }
      })
      
      return await createMenuWindow({
        x: newX,
        y: newY,
        width: 400, // 菜单窗口宽度
        height: 600 // 菜单窗口高度
      })
    } catch (error) {
      console.error('使用主窗口位置创建菜单窗口错误:', error)
      return await createMenuWindow() // 创建默认位置的窗口
    }
  }
}

// 新增：更新菜单窗口位置函数
export async function updateMenuWindowPosition() {
  const menuWindows = Array.from(windowInstances.entries())
    .filter(([key]) => key.startsWith('menu_'))
  
  if (menuWindows.length > 0 && mainWindowPosition) {
    const { x, y, width, height } = mainWindowPosition
    const newX = x + width + 10
    const newY = y
    
    console.log('📱 更新菜单窗口位置:', { newX, newY, mainWindowPosition })

    for (const [windowId, window] of menuWindows) {
      try {
        await window.setPosition(new LogicalPosition(newX, newY))
        console.log('更新菜单窗口位置:', { newX, newY })
      } catch (error) {
        console.error('更新菜单窗口位置失败:', error)
      }
    }
  }
}

/**
 * 检查是否有菜单窗口打开
 */
export function hasMenuWindow() {
  return Array.from(windowInstances.keys()).some(key => key.startsWith('menu_'))
}

/**
 * 实时更新菜单窗口位置（基于当前主窗口位置）
 */
export async function updateMenuWindowPositionRealTime() {
  const menuWindows = Array.from(windowInstances.entries())
    .filter(([key]) => key.startsWith('menu_'))
  
  if (menuWindows.length > 0 && mainWindowPosition) {
    const { x, y, width, height } = mainWindowPosition
    const newX = x + width + 10
    const newY = y
    
    console.log('🔄 实时更新菜单窗口位置:', { newX, newY })
    
    for (const [windowId, window] of menuWindows) {
      try {
        await window.setPosition(new LogicalPosition(newX, newY))
      } catch (error) {
        console.error('❌ 实时更新菜单窗口位置失败:', error)
      }
    }
  }
}

/**
 * 创建剪贴板窗口
 * @param {Object} options 窗口配置
 */
export async function createClipboardWindow(options = {}) {
  // const windowId = `clipboard_${Date.now()}`
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
  console.log('🔍 查找已存在的剪贴板窗口...')
  const clipboardWindows = Array.from(windowInstances.entries())
    .filter(([key]) => key.startsWith('c'))
  console.log('正在查找')
  console.log(clipboardWindows)
  if (clipboardWindows.length > 0) {
    // 如果存在剪贴板窗口，关闭它们
    console.log('存在窗口')
    for (const [windowId, window] of clipboardWindows) {
      try {
        await window.close()
        windowInstances.delete(windowId)
        console.log('关闭窗口成功')
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

/**
 * 关闭所有菜单窗口
 */
export async function closeAllMenuWindows() {
  const menuWindows = Array.from(windowInstances.entries())
    .filter(([key]) => key.startsWith('menu_'))
  
  for (const [windowId, window] of menuWindows) {
    try {
      await window.close()
      windowInstances.delete(windowId)
    } catch (error) {
      console.error('关闭菜单窗口失败:', error)
    }
  }
}


// 将函数暴露给全局，方便 Tauri 调用
if (typeof window !== 'undefined') {
  window.toggleClipboardWindow = toggleClipboardWindow;
  window.toggleMenuWindow = toggleMenuWindow;
  window.updateMenuWindowPosition = updateMenuWindowPosition;
  window.hasMenuWindow = hasMenuWindow;
}