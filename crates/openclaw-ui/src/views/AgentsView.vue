<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useApi } from '@/composables/useApi'

interface AgentInfo {
  agent_id: string
  available: boolean
  config?: {
    model?: string
    system_prompt?: string
    max_tokens?: number
    temperature?: number
  }
}

const api = useApi()

const agents = ref<string[]>([])
const selectedAgent = ref<AgentInfo | null>(null)
const loading = ref(true)
const loadingDetails = ref(false)

onMounted(async () => {
  try {
    const result = await api.call<{ agents: string[] }>('agent.list')
    agents.value = result.agents
  } finally {
    loading.value = false
  }
})

async function selectAgent(agentId: string) {
  loadingDetails.value = true
  try {
    const result = await api.call<AgentInfo>('agent.get', { agent_id: agentId })
    selectedAgent.value = result
  } catch (e) {
    console.error('Failed to load agent details:', e)
  } finally {
    loadingDetails.value = false
  }
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold text-gray-900">Agents</h1>

    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
    </div>

    <div v-else class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <!-- Agent List -->
      <div class="lg:col-span-1">
        <div class="card">
          <div class="card-header">
            <h3 class="text-lg font-medium text-gray-900">Available Agents</h3>
          </div>
          <div class="divide-y divide-gray-200">
            <div v-if="agents.length === 0" class="p-4 text-center text-gray-500">
              No agents configured
            </div>
            <button
              v-for="agentId in agents"
              :key="agentId"
              @click="selectAgent(agentId)"
              :class="[
                'w-full p-4 text-left hover:bg-gray-50 transition-colors',
                selectedAgent?.agent_id === agentId ? 'bg-blue-50' : ''
              ]"
            >
              <div class="font-medium text-gray-900">{{ agentId }}</div>
            </button>
          </div>
        </div>
      </div>

      <!-- Agent Details -->
      <div class="lg:col-span-2">
        <div class="card">
          <div class="card-header">
            <h3 class="text-lg font-medium text-gray-900">Agent Details</h3>
          </div>
          <div class="card-body">
            <div v-if="loadingDetails" class="text-center py-8">
              <div class="inline-block animate-spin rounded-full h-6 w-6 border-4 border-blue-500 border-t-transparent"></div>
            </div>

            <div v-else-if="!selectedAgent" class="text-center py-8 text-gray-500">
              Select an agent to view details
            </div>

            <div v-else class="space-y-4">
              <div class="flex items-center justify-between">
                <h4 class="text-xl font-bold text-gray-900">{{ selectedAgent.agent_id }}</h4>
                <span :class="['badge', selectedAgent.available ? 'badge-success' : 'badge-error']">
                  {{ selectedAgent.available ? 'Available' : 'Unavailable' }}
                </span>
              </div>

              <div v-if="selectedAgent.config" class="space-y-4">
                <div>
                  <label class="text-sm text-gray-500">Model</label>
                  <div class="font-medium text-gray-900">
                    {{ selectedAgent.config.model || 'Not specified' }}
                  </div>
                </div>

                <div>
                  <label class="text-sm text-gray-500">Temperature</label>
                  <div class="font-medium text-gray-900">
                    {{ selectedAgent.config.temperature ?? 'Default' }}
                  </div>
                </div>

                <div>
                  <label class="text-sm text-gray-500">Max Tokens</label>
                  <div class="font-medium text-gray-900">
                    {{ selectedAgent.config.max_tokens ?? 'Default' }}
                  </div>
                </div>

                <div v-if="selectedAgent.config.system_prompt">
                  <label class="text-sm text-gray-500">System Prompt</label>
                  <div class="mt-1 p-3 bg-gray-50 rounded-lg text-sm font-mono whitespace-pre-wrap">
                    {{ selectedAgent.config.system_prompt }}
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
