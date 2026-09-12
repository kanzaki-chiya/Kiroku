import { createRouter, createWebHashHistory, type RouteRecordRaw } from 'vue-router'
import LibraryView from '../views/LibraryView.vue'
import AnimeDetailView from '../views/AnimeDetailView.vue'
import AddEntryView from '../views/AddEntryView.vue'
import EditEntryView from '../views/EditEntryView.vue'
import InsightsView from '../views/InsightsView.vue'
import SettingsView from '../views/SettingsView.vue'
import NotFoundView from '../views/NotFoundView.vue'

export const routes: RouteRecordRaw[] = [
    { path: '/', redirect: '/library' },
    { path: '/library', name: 'library', component: LibraryView },
    { path: '/anime/:id', name: 'anime-detail', component: AnimeDetailView },
    { path: '/add', name: 'add', component: AddEntryView },
    { path: '/anime/:id/edit', name: 'anime-edit', component: EditEntryView },
    { path: '/insights', name: 'insights', component: InsightsView },
    { path: '/settings', name: 'settings', component: SettingsView },
    { path: '/:pathMatch(.*)*', name: 'not-found', component: NotFoundView }
]

export const router = createRouter({
  history: createWebHashHistory(),
  routes,
  scrollBehavior() {
    return { top: 0 }
  }
})
