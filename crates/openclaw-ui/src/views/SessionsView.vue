<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useSessionsStore, type Session } from '@/stores/sessions'

const router = useRouter()
const sessionsStore = useSessionsStore()

const filter = ref({
  channel: '',
  agent: '',
  state: '',
})
const searchQuery = ref('')

onMounted(() => {
  sessionsStore.fetchSessions()
})

async function applyFilter() {
  await sessionsStore.fetchSessions({
    channel: filter.value.channel || undefined,
    agent: filter.value.agent || undefined,
    state: filter.value.state || undefined,
  })
}

async function search() {
  if (searchQuery.value.trim()) {
    const results = await sessionsStore.searchSessions(searchQuery.value)
    sessionsStore.sessions = results
  } else {
    applyFilter()
  }
}

function viewSession(session: Session) {
  router.push(`/sessions/${encodeURIComponent(session.session_key)}`)
}

function formatDate(date: string): string {
  return new Date(date).toLocaleString()
}

function getStateBadgeClass(state: string): string {
  switch (state) {
    case 'active': return 'badge-success'
    case 'paused': return 'badge-warning'
    case 'ended': return 'badge-gray'
    default: return 'badge-info'
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <h1 class="text-2xl font-bold text-gray-900">Sessions</h1>
    </div>

    <!-- Filters -->
    <div class="card">
      <div class="card-body">
        <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
          <div>
            <input
              v-model="searchQuery"
              type="text"
              class="input"
              placeholder="Search sessions..."
              @keyup.enter="search"
            />
          </div>
          <div>
            <select v-model="filter.state" class="input" @change="applyFilter">
              <option value="">All States</option>
              <option value="active">Active</option>
              <option value="paused">Paused</option>
              <option value="ended">Ended</option>
            </select>
          </div>
          <div>
            <input
              v-model="filter.channel"
              type="text"
              class="input"
              placeholder="Filter by channel..."
              @keyup.enter="applyFilter"
            />
          </div>
          <div>
            <input
              v-model="filter.agent"
              type="text"
              class="input"
              placeholder="Filter by agent..."
              @keyup.enter="applyFilter"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- Sessions List -->
    <div class="card">
      <div v-if="sessionsStore.loading" class="card-body text-center py-12">
        <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
      </div>

      <div v-else-if="sessionsStore.sessions.length === 0" class="card-body text-center py-12 text-gray-500">
        No sessions found
      </div>

      <div v-else class="divide-y divide-gray-200">
        <div
          v-for="session in sessionsStore.sessions"
          :key="session.session_key"
          class="p-4 hover:bg-gray-50 cursor-pointer transition-colors"
          @click="viewSession(session)"
        >
          <div class="flex items-center justify-between">
            <div>
              <div class="font-medium text-gray-900">
                {{ session.peer_id }}
              </div>
              <div class="text-sm text-gray-500 mt-1">
                <span class="badge badge-info">{{ session.channel }}</span>
                <span class="mx-2">|</span>
                <span>{{ session.agent_id }}</span>
                <span class="mx-2">|</span>
                <span>{{ session.message_count }} messages</span>
              </div>
            </div>
            <div class="text-right">
              <span :class="['badge', getStateBadgeClass(session.state)]">
                {{ session.state }}
              </span>
              <div class="text-xs text-gray-500 mt-1">
                {{ formatDate(session.last_activity) }}
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
