<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSessionsStore, type SessionStats } from '@/stores/sessions'
import { useWebSocket } from '@/composables/useWebSocket'
import { useApi } from '@/composables/useApi'

const sessionsStore = useSessionsStore()
const { events, connected } = useWebSocket()
const api = useApi()

const stats = ref<SessionStats | null>(null)
const systemHealth = ref<Record<string, unknown> | null>(null)
const systemVersion = ref<{ version: string; name: string } | null>(null)
const loading = ref(true)

onMounted(async () => {
  try {
    const [statsResult, healthResult, versionResult] = await Promise.all([
      sessionsStore.fetchStats(),
      api.call<Record<string, unknown>>('system.health'),
      api.call<{ version: string; name: string }>('system.version'),
    ])
    stats.value = statsResult
    systemHealth.value = healthResult
    systemVersion.value = versionResult
  } finally {
    loading.value = false
  }
})

function formatEventType(event: Record<string, unknown>): string {
  const type = event.type as string || 'unknown'
  return type.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase())
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold text-gray-900">Dashboard</h1>

    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
      <p class="mt-2 text-gray-600">Loading...</p>
    </div>

    <template v-else>
      <!-- Stats Cards -->
      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div class="card">
          <div class="card-body">
            <div class="text-sm font-medium text-gray-500">Total Sessions</div>
            <div class="text-3xl font-bold text-gray-900 mt-1">
              {{ stats?.total || 0 }}
            </div>
          </div>
        </div>

        <div class="card">
          <div class="card-body">
            <div class="text-sm font-medium text-gray-500">Active Sessions</div>
            <div class="text-3xl font-bold text-green-600 mt-1">
              {{ stats?.active || 0 }}
            </div>
          </div>
        </div>

        <div class="card">
          <div class="card-body">
            <div class="text-sm font-medium text-gray-500">Total Messages</div>
            <div class="text-3xl font-bold text-gray-900 mt-1">
              {{ stats?.total_messages || 0 }}
            </div>
          </div>
        </div>

        <div class="card">
          <div class="card-body">
            <div class="text-sm font-medium text-gray-500">System Status</div>
            <div class="mt-1">
              <span :class="['badge', systemHealth?.status === 'healthy' ? 'badge-success' : 'badge-error']">
                {{ systemHealth?.status || 'unknown' }}
              </span>
            </div>
            <div class="text-xs text-gray-500 mt-2">
              {{ systemVersion?.name }} v{{ systemVersion?.version }}
            </div>
          </div>
        </div>
      </div>

      <!-- By Channel & Agent -->
      <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <div class="card">
          <div class="card-header">
            <h3 class="text-lg font-medium text-gray-900">Sessions by Channel</h3>
          </div>
          <div class="card-body">
            <div v-if="stats?.by_channel && Object.keys(stats.by_channel).length > 0" class="space-y-2">
              <div v-for="(count, channel) in stats.by_channel" :key="channel" class="flex justify-between items-center">
                <span class="text-gray-600">{{ channel }}</span>
                <span class="font-medium text-gray-900">{{ count }}</span>
              </div>
            </div>
            <div v-else class="text-gray-500 text-sm">No sessions yet</div>
          </div>
        </div>

        <div class="card">
          <div class="card-header">
            <h3 class="text-lg font-medium text-gray-900">Sessions by Agent</h3>
          </div>
          <div class="card-body">
            <div v-if="stats?.by_agent && Object.keys(stats.by_agent).length > 0" class="space-y-2">
              <div v-for="(count, agent) in stats.by_agent" :key="agent" class="flex justify-between items-center">
                <span class="text-gray-600">{{ agent }}</span>
                <span class="font-medium text-gray-900">{{ count }}</span>
              </div>
            </div>
            <div v-else class="text-gray-500 text-sm">No sessions yet</div>
          </div>
        </div>
      </div>

      <!-- Real-time Events -->
      <div class="card">
        <div class="card-header flex justify-between items-center">
          <h3 class="text-lg font-medium text-gray-900">Real-time Events</h3>
          <span :class="['badge', connected ? 'badge-success' : 'badge-error']">
            {{ connected ? 'Live' : 'Disconnected' }}
          </span>
        </div>
        <div class="card-body max-h-96 overflow-y-auto">
          <div v-if="events.length > 0" class="space-y-2">
            <div
              v-for="envelope in events.slice(0, 20)"
              :key="envelope.id"
              class="p-3 bg-gray-50 rounded-lg text-sm"
            >
              <div class="flex justify-between items-start">
                <span class="font-medium text-gray-900">
                  {{ formatEventType(envelope.event) }}
                </span>
                <span class="text-xs text-gray-500">
                  {{ new Date(envelope.timestamp).toLocaleTimeString() }}
                </span>
              </div>
              <div class="text-gray-600 mt-1 text-xs font-mono">
                {{ JSON.stringify(envelope.event).slice(0, 100) }}...
              </div>
            </div>
          </div>
          <div v-else class="text-gray-500 text-center py-8">
            No events yet. Activity will appear here in real-time.
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
