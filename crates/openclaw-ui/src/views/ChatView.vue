<script setup lang="ts">
import { ref, nextTick, onMounted } from 'vue'
import { useSessionsStore } from '@/stores/sessions'
import { useApi } from '@/composables/useApi'

interface Message {
  role: 'user' | 'assistant'
  content: string
}

const sessionsStore = useSessionsStore()
const api = useApi()

const sessionKey = ref<string | null>(null)
const agentId = ref('default')
const agents = ref<string[]>([])
const messages = ref<Message[]>([])
const inputMessage = ref('')
const loading = ref(false)
const messagesContainer = ref<HTMLElement | null>(null)

onMounted(async () => {
  try {
    const result = await api.call<{ agents: string[] }>('agent.list')
    agents.value = result.agents
    const firstAgent = agents.value[0]
    if (firstAgent && !agents.value.includes(agentId.value)) {
      agentId.value = firstAgent
    }
  } catch (e) {
    console.error('Failed to load agents:', e)
  }
})

async function scrollToBottom() {
  await nextTick()
  if (messagesContainer.value) {
    messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
  }
}

async function sendMessage() {
  if (!inputMessage.value.trim() || loading.value) return

  const userMessage = inputMessage.value.trim()
  inputMessage.value = ''

  // Create session if needed
  if (!sessionKey.value) {
    try {
      sessionKey.value = await sessionsStore.createSession(agentId.value, 'ui', 'web-user')
    } catch (e) {
      console.error('Failed to create session:', e)
      return
    }
  }

  // Add user message
  messages.value.push({ role: 'user', content: userMessage })
  await scrollToBottom()

  // Send to agent
  loading.value = true
  try {
    const response = await sessionsStore.sendMessage(sessionKey.value, userMessage, agentId.value)
    messages.value.push({ role: 'assistant', content: response })
    await scrollToBottom()
  } catch (e) {
    messages.value.push({
      role: 'assistant',
      content: `Error: ${e instanceof Error ? e.message : 'Failed to get response'}`
    })
    await scrollToBottom()
  } finally {
    loading.value = false
  }
}

function startNewChat() {
  sessionKey.value = null
  messages.value = []
}
</script>

<template>
  <div class="flex flex-col h-[calc(100vh-8rem)]">
    <!-- Header -->
    <div class="flex items-center justify-between mb-4">
      <h1 class="text-2xl font-bold text-gray-900">Chat</h1>
      <div class="flex items-center space-x-4">
        <select
          v-model="agentId"
          class="input w-48"
          :disabled="sessionKey !== null"
        >
          <option v-for="agent in agents" :key="agent" :value="agent">
            {{ agent }}
          </option>
        </select>
        <button
          @click="startNewChat"
          class="btn btn-secondary"
          :disabled="messages.length === 0"
        >
          New Chat
        </button>
      </div>
    </div>

    <!-- Messages -->
    <div
      ref="messagesContainer"
      class="flex-1 overflow-y-auto card"
    >
      <div class="p-4 space-y-4">
        <div v-if="messages.length === 0" class="text-center py-12 text-gray-500">
          Start a conversation with the AI agent
        </div>

        <div
          v-for="(message, index) in messages"
          :key="index"
          :class="[
            'flex',
            message.role === 'user' ? 'justify-end' : 'justify-start'
          ]"
        >
          <div
            :class="[
              'max-w-[70%] rounded-lg px-4 py-2',
              message.role === 'user'
                ? 'bg-blue-600 text-white'
                : 'bg-gray-100 text-gray-900'
            ]"
          >
            <div class="whitespace-pre-wrap">{{ message.content }}</div>
          </div>
        </div>

        <div v-if="loading" class="flex justify-start">
          <div class="bg-gray-100 rounded-lg px-4 py-2">
            <div class="flex space-x-2">
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce"></div>
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0.1s"></div>
              <div class="w-2 h-2 bg-gray-400 rounded-full animate-bounce" style="animation-delay: 0.2s"></div>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Input -->
    <div class="mt-4">
      <form @submit.prevent="sendMessage" class="flex space-x-4">
        <input
          v-model="inputMessage"
          type="text"
          class="input flex-1"
          placeholder="Type your message..."
          :disabled="loading"
        />
        <button
          type="submit"
          class="btn btn-primary"
          :disabled="loading || !inputMessage.trim()"
        >
          Send
        </button>
      </form>
    </div>
  </div>
</template>
