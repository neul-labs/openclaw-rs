<script setup lang="ts">
import { RouterLink, useRoute } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { computed } from 'vue'

const route = useRoute()
const authStore = useAuthStore()

const navItems = computed(() => {
  const items = [
    { name: 'Dashboard', path: '/', icon: 'home' },
    { name: 'Sessions', path: '/sessions', icon: 'chat' },
    { name: 'Chat', path: '/chat', icon: 'message' },
    { name: 'Agents', path: '/agents', icon: 'robot' },
    { name: 'Channels', path: '/channels', icon: 'plug' },
    { name: 'Tools', path: '/tools', icon: 'wrench' },
  ]

  if (authStore.isAdmin) {
    items.push({ name: 'Admin', path: '/admin', icon: 'settings' })
  }

  return items
})

function isActive(path: string): boolean {
  if (path === '/') {
    return route.path === '/'
  }
  return route.path.startsWith(path)
}
</script>

<template>
  <aside class="w-64 bg-gray-800 text-white flex-shrink-0">
    <nav class="py-4">
      <ul class="space-y-1">
        <li v-for="item in navItems" :key="item.path">
          <RouterLink
            :to="item.path"
            :class="[
              'flex items-center px-6 py-3 text-sm font-medium transition-colors',
              isActive(item.path)
                ? 'bg-gray-900 text-white border-l-4 border-blue-500'
                : 'text-gray-300 hover:bg-gray-700 hover:text-white'
            ]"
          >
            <span>{{ item.name }}</span>
          </RouterLink>
        </li>
      </ul>
    </nav>

    <div class="absolute bottom-0 left-0 w-64 p-4 border-t border-gray-700">
      <div class="text-xs text-gray-500">
        OpenClaw Gateway v0.1.0
      </div>
    </div>
  </aside>
</template>
