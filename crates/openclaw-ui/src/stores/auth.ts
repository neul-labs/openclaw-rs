import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { useApi } from '@/composables/useApi'

export interface User {
  id: string
  username: string
  role: 'admin' | 'operator' | 'viewer'
  email?: string
  active: boolean
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem('openclaw_token'))
  const refreshToken = ref<string | null>(localStorage.getItem('openclaw_refresh_token'))
  const user = ref<User | null>(null)
  const setupRequired = ref(false)
  const setupChecked = ref(false)

  const isAuthenticated = computed(() => !!token.value && !!user.value)
  const isAdmin = computed(() => user.value?.role === 'admin')

  async function checkSetupStatus(): Promise<boolean> {
    if (setupChecked.value) {
      return setupRequired.value
    }

    try {
      const api = useApi()
      const result = await api.call<{ needs_setup: boolean }>('setup.status')
      setupRequired.value = result.needs_setup
      setupChecked.value = true
      return result.needs_setup
    } catch {
      setupChecked.value = true
      return false
    }
  }

  async function login(username: string, password: string): Promise<void> {
    const api = useApi()
    const result = await api.call<{
      token: string
      refresh_token: string
      user: User
    }>('auth.login', { username, password })

    token.value = result.token
    refreshToken.value = result.refresh_token
    user.value = result.user

    localStorage.setItem('openclaw_token', result.token)
    localStorage.setItem('openclaw_refresh_token', result.refresh_token)
  }

  async function logout(): Promise<void> {
    try {
      const api = useApi()
      await api.call('auth.logout')
    } finally {
      token.value = null
      refreshToken.value = null
      user.value = null
      localStorage.removeItem('openclaw_token')
      localStorage.removeItem('openclaw_refresh_token')
    }
  }

  async function fetchCurrentUser(): Promise<void> {
    if (!token.value) return
    try {
      const api = useApi()
      const result = await api.call<{ user: User }>('auth.me')
      user.value = result.user
    } catch {
      // Token invalid, clear
      token.value = null
      refreshToken.value = null
      user.value = null
      localStorage.removeItem('openclaw_token')
      localStorage.removeItem('openclaw_refresh_token')
    }
  }

  async function setup(
    bootstrapToken: string,
    adminUsername: string,
    adminPassword: string,
    email?: string
  ): Promise<void> {
    const api = useApi()
    const result = await api.call<{
      token: string
      refresh_token: string
      user: User
    }>('setup.init', {
      bootstrap_token: bootstrapToken,
      admin_username: adminUsername,
      admin_password: adminPassword,
      email,
    })

    token.value = result.token
    refreshToken.value = result.refresh_token
    user.value = result.user
    setupRequired.value = false

    localStorage.setItem('openclaw_token', result.token)
    localStorage.setItem('openclaw_refresh_token', result.refresh_token)
  }

  return {
    token,
    refreshToken,
    user,
    setupRequired,
    isAuthenticated,
    isAdmin,
    checkSetupStatus,
    login,
    logout,
    fetchCurrentUser,
    setup,
  }
})
