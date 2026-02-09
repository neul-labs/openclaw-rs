<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { useSessionsStore, type Session } from '@/stores/sessions'

const props = defineProps<{
  key?: string
}>()

const route = useRoute()
const router = useRouter()
const sessionsStore = useSessionsStore()

const session = ref<Session | null>(null)
const loading = ref(true)
const error = ref('')

const sessionKey = computed(() => props.key || route.params.key as string)

onMounted(async () => {
  try {
    session.value = await sessionsStore.fetchSession(decodeURIComponent(sessionKey.value))
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to load session'
  } finally {
    loading.value = false
  }
})

async function endSession() {
  if (!session.value) return
  if (!confirm('Are you sure you want to end this session?')) return

  try {
    await sessionsStore.endSession(session.value.session_key)
    session.value.state = 'ended'
  } catch (e) {
    error.value = e instanceof Error ? e.message : 'Failed to end session'
  }
}

function getMessageClass(type: string): string {
  switch (type) {
    case 'inbound': return 'bg-blue-50 border-blue-200'
    case 'outbound': return 'bg-green-50 border-green-200'
    case 'tool': return 'bg-yellow-50 border-yellow-200'
    default: return 'bg-gray-50 border-gray-200'
  }
}

function getMessageLabel(type: string): string {
  switch (type) {
    case 'inbound': return 'User'
    case 'outbound': return 'Agent'
    case 'tool': return 'Tool'
    default: return type
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="flex items-center justify-between">
      <div class="flex items-center space-x-4">
        <button @click="router.back()" class="btn btn-secondary">
          Back
        </button>
        <h1 class="text-2xl font-bold text-gray-900">Session Details</h1>
      </div>
      <button
        v-if="session?.state === 'active'"
        @click="endSession"
        class="btn btn-danger"
      >
        End Session
      </button>
    </div>

    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
    </div>

    <div v-else-if="error" class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg">
      {{ error }}
    </div>

    <template v-else-if="session">
      <!-- Session Info -->
      <div class="card">
        <div class="card-header">
          <h3 class="text-lg font-medium text-gray-900">Session Information</h3>
        </div>
        <div class="card-body">
          <dl class="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div>
              <dt class="text-sm text-gray-500">Peer ID</dt>
              <dd class="font-medium text-gray-900">{{ session.peer_id }}</dd>
            </div>
            <div>
              <dt class="text-sm text-gray-500">Channel</dt>
              <dd class="font-medium text-gray-900">{{ session.channel }}</dd>
            </div>
            <div>
              <dt class="text-sm text-gray-500">Agent</dt>
              <dd class="font-medium text-gray-900">{{ session.agent_id }}</dd>
            </div>
            <div>
              <dt class="text-sm text-gray-500">State</dt>
              <dd>
                <span :class="[
                  'badge',
                  session.state === 'active' ? 'badge-success' :
                  session.state === 'paused' ? 'badge-warning' : 'badge-gray'
                ]">
                  {{ session.state }}
                </span>
              </dd>
            </div>
            <div>
              <dt class="text-sm text-gray-500">Created</dt>
              <dd class="font-medium text-gray-900">{{ new Date(session.created_at).toLocaleString() }}</dd>
            </div>
            <div>
              <dt class="text-sm text-gray-500">Last Activity</dt>
              <dd class="font-medium text-gray-900">{{ new Date(session.last_activity).toLocaleString() }}</dd>
            </div>
            <div>
              <dt class="text-sm text-gray-500">Message Count</dt>
              <dd class="font-medium text-gray-900">{{ session.message_count }}</dd>
            </div>
          </dl>
        </div>
      </div>

      <!-- Messages -->
      <div class="card">
        <div class="card-header">
          <h3 class="text-lg font-medium text-gray-900">Messages</h3>
        </div>
        <div class="card-body max-h-[600px] overflow-y-auto">
          <div v-if="session.messages.length === 0" class="text-center py-8 text-gray-500">
            No messages in this session
          </div>
          <div v-else class="space-y-4">
            <div
              v-for="(message, index) in session.messages"
              :key="index"
              :class="['p-4 rounded-lg border', getMessageClass(message.type)]"
            >
              <div class="flex items-center justify-between mb-2">
                <span class="font-medium text-sm">{{ getMessageLabel(message.type) }}</span>
                <span v-if="message.tool_name" class="text-xs text-gray-500">
                  {{ message.tool_name }}
                </span>
              </div>
              <div class="whitespace-pre-wrap text-gray-900">{{ message.content }}</div>
            </div>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
