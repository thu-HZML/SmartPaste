import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue' // 这个 App.vue 将作为根组件
import router from './router'

// 创建 Pinia 实例
const pinia = createPinia()

// 创建 Vue 应用并依次使用插件
const app = createApp(App)

app.use(pinia)    // 先使用 Pinia
app.use(router)   // 再使用路由

app.mount('#app')
