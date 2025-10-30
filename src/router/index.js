import { createRouter, createWebHistory } from 'vue-router'
import ClipboardApp from '../components/ClipboardApp.vue'
import Settings from '../components/Settings.vue'

const routes = [
  {
    path: '/',
    name: 'Home',
    component: ClipboardApp
  },
  {
    path: '/settings',
    name: 'Settings',
    component: Settings
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router