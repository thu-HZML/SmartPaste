import { createRouter, createWebHistory } from 'vue-router'
import ClipboardApp from '../components/ClipboardApp.vue'
import Settings from '../components/Settings.vue'
import Preferences from '../components/Preferences.vue'
import DesktopPet from '../components/DesktopPet.vue'  // 新增桌宠组件

const routes = [
  {
    path: '/',
    name: 'DesktopPet',
    component: DesktopPet
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Settings
  },
  {
    path: '/preferences',
    name: 'Preferences',
    component: Preferences
  },
  {
    path: '/clipboardapp',
    name: 'ClipboardApp',
    component: ClipboardApp
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router