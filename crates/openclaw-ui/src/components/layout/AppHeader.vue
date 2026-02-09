<script setup lang="ts">
import { useAuthStore } from '@/stores/auth'
import { useWebSocket } from '@/composables/useWebSocket'
import { useRouter } from 'vue-router'

const authStore = useAuthStore()
const { connected } = useWebSocket()
const router = useRouter()

async function handleLogout() {
  await authStore.logout()
  router.push('/login')
}
</script>

<template>
  <header class="bg-white border-b border-gray-200 px-6 py-4">
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-4">
        <h1 class="text-xl font-bold text-gray-900">OpenClaw</h1>
        <span
          :class="[
            'badge',
            connected ? 'badge-success' : 'badge-error'
          ]"
        >
          {{ connected ? 'Connected' : 'Disconnected' }}
        </span>
      </div>

      <div class="flex items-center space-x-4">
        <span v-if="authStore.user" class="text-sm text-gray-600">
          {{ authStore.user.username }}
          <span class="badge badge-info ml-2">{{ authStore.user.role }}</span>
        </span>
        <button
          @click="handleLogout"
          class="btn btn-secondary text-sm"
        >
          Logout
        </button>
      </div>
    </div>
  </header>
</template>
