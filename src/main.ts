import { createApp } from 'vue'
import { createPinia } from 'pinia'
import App from './App.vue'
import { router } from './router'
import { initTheme } from './services/theme'
import { useLibraryStore } from './stores/library'
import './styles/main.css'

initTheme()

async function start() {
  const app = createApp(App)
  const pinia = createPinia()
  app.use(pinia)
  app.use(router)
  const library = useLibraryStore()
  try {
    await library.hydrate()
    await library.listenCovers()
  } catch {
    // bootError is rendered by App.vue
  }
  app.mount('#app')
}

void start()
