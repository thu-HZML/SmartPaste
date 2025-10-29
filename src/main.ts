import { createApp } from 'vue'
import App from './App.vue' // 这个 App.vue 将作为根组件
import router from './router'

createApp(App).use(router).mount('#app')