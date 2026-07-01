import { defineStore } from 'pinia'
import { ref } from 'vue'
import { clearToken, getToken, setToken } from '@/lib/api'

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(getToken())

  function saveToken(value: string) {
    token.value = value
    setToken(value)
  }

  function logout() {
    token.value = null
    clearToken()
  }

  const isLoggedIn = () => !!token.value

  return { token, saveToken, logout, isLoggedIn }
})
