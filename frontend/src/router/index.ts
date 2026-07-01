import { createRouter, createWebHistory } from 'vue-router'
import { clearToken, isTokenValid } from '@/lib/api'

const router = createRouter({
  history: createWebHistory(),
  routes: [
    {
      path: '/login',
      name: 'login',
      component: () => import('@/views/LoginView.vue'),
      meta: { guest: true },
    },
    {
      path: '/',
      name: 'dashboard',
      component: () => import('@/views/DashboardView.vue'),
      meta: { requiresAuth: true },
    },
  ],
})

router.beforeEach((to) => {
  if (!isTokenValid()) {
    clearToken()
  }
  const loggedIn = isTokenValid()
  if (to.meta.requiresAuth && !loggedIn) {
    return { name: 'login' }
  }
  if (to.meta.guest && loggedIn) {
    return { name: 'dashboard' }
  }
})

export default router
