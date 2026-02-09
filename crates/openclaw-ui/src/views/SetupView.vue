<script setup lang="ts">
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'

const router = useRouter()
const authStore = useAuthStore()

const bootstrapToken = ref('')
const username = ref('')
const password = ref('')
const confirmPassword = ref('')
const email = ref('')
const error = ref('')
const loading = ref(false)

async function handleSetup() {
  error.value = ''

  if (password.value !== confirmPassword.value) {
    error.value = 'Passwords do not match'
    return
  }

  if (password.value.length < 8) {
    error.value = 'Password must be at least 8 characters'
    return
  }

  loading.value = true

  try {
    await authStore.setup(
      bootstrapToken.value,
      username.value,
      password.value,
      email.value || undefined
    )
    router.push('/')
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Setup failed'
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
            OpenClaw Setup
          </h2>
          <p class="text-sm text-gray-600 text-center mt-2">
            Create your admin account to get started
          </p>
        </div>

        <div class="card-body">
          <form @submit.prevent="handleSetup" class="space-y-4">
            <div v-if="error" class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
              {{ error }}
            </div>

            <div>
              <label for="bootstrap-token" class="label">Bootstrap Token</label>
              <input
                id="bootstrap-token"
                v-model="bootstrapToken"
                type="text"
                class="input font-mono"
                placeholder="Check the server logs for this token"
                required
              />
              <p class="text-xs text-gray-500 mt-1">
                Find this token in the gateway server startup logs
              </p>
            </div>

            <div>
              <label for="username" class="label">Admin Username</label>
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
              <label for="email" class="label">Email (optional)</label>
              <input
                id="email"
                v-model="email"
                type="email"
                class="input"
                autocomplete="email"
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
                autocomplete="new-password"
                minlength="8"
              />
            </div>

            <div>
              <label for="confirm-password" class="label">Confirm Password</label>
              <input
                id="confirm-password"
                v-model="confirmPassword"
                type="password"
                class="input"
                required
                autocomplete="new-password"
              />
            </div>

            <button
              type="submit"
              class="btn btn-primary w-full"
              :disabled="loading"
            >
              {{ loading ? 'Setting up...' : 'Complete Setup' }}
            </button>
          </form>
        </div>
      </div>
    </div>
  </div>
</template>
