// src/components/MenuFunctions.js
import { toggleClipboardWindow, toggleFavoritesWindow } from '../utils/actions.js'

export function useMenuFunctions() {
  const openHistory = async () => {
    console.log('打开历史记录 - 调用 toggleClipboardWindow')
    try {
      await toggleClipboardWindow()
      console.log('📋 剪贴板窗口已切换')
    } catch (error) {
      console.error('切换剪贴板窗口失败:', error)
    }
  }

  const openFavorites = async () => {
    console.log('打开收藏夹')
    try {
      await toggleFavoritesWindow()
      console.log('⭐ 收藏夹窗口已打开')
    } catch (error) {
      console.error('打开收藏夹窗口失败:', error)
    }
  }

  const openSettings = async () => {
    console.log('打开设置')
    try {
      await toggleSetWindow()
      console.log('设置窗口已打开')
    } catch (error) {
      console.error('打开设置窗口失败:', error)
    }
  }

  const openCloud = () => {
    console.log('打开云端')
  }

  const openAIAssistant = () => {
    console.log('打开AI助手')
  }

  const openHelp = () => {
    console.log('打开帮助')
  }

  return {
    openHistory,
    openFavorites,
    openSettings,
    openCloud,
    openAIAssistant,
    openHelp
  }
}