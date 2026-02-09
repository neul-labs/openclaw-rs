<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useApi } from '@/composables/useApi'

interface ChannelStatus {
  connected: boolean
  account_id?: string
  display_name?: string
  error?: string
}

const api = useApi()

const channels = ref<string[]>([])
const statuses = ref<Record<string, ChannelStatus>>({})
const loading = ref(true)
const refreshing = ref(false)

onMounted(async () => {
  await loadChannels()
})

async function loadChannels() {
  try {
    const [channelsResult, statusResult] = await Promise.all([
      api.call<{ channels: string[] }>('channels.list'),
      api.call<{ statuses: Record<string, ChannelStatus> }>('channels.status'),
    ])
    channels.value = channelsResult.channels
    statuses.value = statusResult.statuses
  } finally {
    loading.value = false
  }
}

async function refreshStatuses() {
  refreshing.value = true
  try {
    const result = await api.call<{ statuses: Record<string, ChannelStatus> }>('channels.status')
    statuses.value = result.statuses
  } finally {
    refreshing.value = false
  }
}

async function probeChannel(channelId: string) {
  try {
    const result = await api.call<ChannelStatus>('channels.probe', { channel_id: channelId })
    statuses.value[channelId] = result
  } catch (e) {
    statuses.value[channelId] = {
      connected: false,
      error: e instanceof Error ? e.message : 'Probe failed',
    }
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold text-gray-900">Channels</h1>
      <button
        @click="refreshStatuses"
        class="btn btn-secondary"
        :disabled="refreshing"
      >
        {{ refreshing ? 'Refreshing...' : 'Refresh Status' }}
      </button>
    </div>

    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
    </div>

    <div v-else-if="channels.length === 0" class="card">
      <div class="card-body text-center py-12 text-gray-500">
        No channels configured
      </div>
    </div>

    <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
      <div v-for="channel in channels" :key="channel" class="card">
        <div class="card-header flex items-center justify-between">
          <h3 class="text-lg font-medium text-gray-900">{{ channel }}</h3>
          <span :class="[
            'badge',
            statuses[channel]?.connected ? 'badge-success' : 'badge-error'
          ]">
            {{ statuses[channel]?.connected ? 'Connected' : 'Disconnected' }}
          </span>
        </div>
        <div class="card-body">
          <dl class="space-y-2">
            <div v-if="statuses[channel]?.account_id">
              <dt class="text-sm text-gray-500">Account ID</dt>
              <dd class="font-medium text-gray-900">{{ statuses[channel].account_id }}</dd>
            </div>
            <div v-if="statuses[channel]?.display_name">
              <dt class="text-sm text-gray-500">Display Name</dt>
              <dd class="font-medium text-gray-900">{{ statuses[channel].display_name }}</dd>
            </div>
            <div v-if="statuses[channel]?.error">
              <dt class="text-sm text-gray-500">Error</dt>
              <dd class="text-red-600 text-sm">{{ statuses[channel].error }}</dd>
            </div>
          </dl>

          <button
            @click="probeChannel(channel)"
            class="btn btn-secondary w-full mt-4"
          >
            Probe Channel
          </button>
        </div>
      </div>
    </div>
  </div>
</template>
