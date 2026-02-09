<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()

const username = ref('')
const password = ref('')
const error = ref('')
const loading = ref(false)

async function handleLogin() {
  error.value = ''
  loading.value = true

  try {
    await authStore.login(username.value, password.value)
    const redirect = route.query.redirect as string || '/'
    router.push(redirect)
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Login failed'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="min-h-screen flex items-center justify-center bg-gray-50">
    <div class="max-w-md w-full">
      <div class="card">
        <div class="card-header">
          <h2 class="text-2xl font-bold text-center text-gray-900">
            OpenClaw Login
          </h2>
        </div>

        <div class="card-body">
          <form @submit.prevent="handleLogin" class="space-y-4">
            <div v-if="error" class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
              {{ error }}
            </div>

            <div>
              <label for="username" class="label">Username</label>
              <input
                id="username"
                v-model="username"
                type="text"
                class="input"
                required
                autocomplete="username"
              />
            </div>

            <div>
              <label for="password" class="label">Password</label>
              <input
                id="password"
                v-model="password"
                type="password"
                class="input"
                required
                autocomplete="current-password"
              />
            </div>

            <button
              type="submit"
              class="btn btn-primary w-full"
              :disabled="loading"
            >
              {{ loading ? 'Signing in...' : 'Sign in' }}
            </button>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>
