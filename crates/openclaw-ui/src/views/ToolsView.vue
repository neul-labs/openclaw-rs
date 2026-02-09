<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useApi } from '@/composables/useApi'

interface Tool {
  name: string
  description: string
  input_schema: Record<string, unknown>
}

const api = useApi()

const tools = ref<Tool[]>([])
const selectedTool = ref<Tool | null>(null)
const toolParams = ref<Record<string, string>>({})
const executionResult = ref<unknown>(null)
const loading = ref(true)
const executing = ref(false)

onMounted(async () => {
  try {
    const result = await api.call<{ tools: Tool[] }>('tools.list')
    tools.value = result.tools
  } finally {
    loading.value = false
  }
})

function selectTool(tool: Tool) {
  selectedTool.value = tool
  toolParams.value = {}
  executionResult.value = null

  // Initialize params from schema
  const props = tool.input_schema?.properties as Record<string, { type: string }> | undefined
  if (props) {
    for (const key of Object.keys(props)) {
      toolParams.value[key] = ''
    }
  }
}

async function executeTool() {
  if (!selectedTool.value) return

  executing.value = true
  executionResult.value = null

  try {
    // Parse params - convert strings to appropriate types
    const params: Record<string, unknown> = {}
    for (const [key, value] of Object.entries(toolParams.value)) {
      if (value.trim()) {
        // Try to parse as JSON first
        try {
          params[key] = JSON.parse(value)
        } catch {
          params[key] = value
        }
      }
    }

    const result = await api.call('tools.execute', {
      tool_name: selectedTool.value.name,
      params,
    })
    executionResult.value = result
  } catch (e) {
    executionResult.value = { error: e instanceof Error ? e.message : 'Execution failed' }
  } finally {
    executing.value = false
  }
}

function getPropertyType(schema: Record<string, unknown>, propName: string): string {
  const props = schema?.properties as Record<string, { type?: string }> | undefined
  return props?.[propName]?.type || 'string'
}

function isRequired(schema: Record<string, unknown>, propName: string): boolean {
  const required = schema?.required as string[] | undefined
  return required?.includes(propName) || false
}
</script>

<template>
  <div class="space-y-6">
    <h1 class="text-2xl font-bold text-gray-900">Tools</h1>

    <div v-if="loading" class="text-center py-12">
      <div class="inline-block animate-spin rounded-full h-8 w-8 border-4 border-blue-500 border-t-transparent"></div>
    </div>

    <div v-else class="grid grid-cols-1 lg:grid-cols-3 gap-6">
      <!-- Tool List -->
      <div class="lg:col-span-1">
        <div class="card">
          <div class="card-header">
            <h3 class="text-lg font-medium text-gray-900">Available Tools</h3>
          </div>
          <div class="divide-y divide-gray-200 max-h-[600px] overflow-y-auto">
            <div v-if="tools.length === 0" class="p-4 text-center text-gray-500">
              No tools registered
            </div>
            <button
              v-for="tool in tools"
              :key="tool.name"
              @click="selectTool(tool)"
              :class="[
                'w-full p-4 text-left hover:bg-gray-50 transition-colors',
                selectedTool?.name === tool.name ? 'bg-blue-50' : ''
              ]"
            >
              <div class="font-medium text-gray-900">{{ tool.name }}</div>
              <div class="text-sm text-gray-500 mt-1 line-clamp-2">{{ tool.description }}</div>
            </button>
          </div>
        </div>
      </div>

      <!-- Tool Details & Execution -->
      <div class="lg:col-span-2 space-y-6">
        <div class="card">
          <div class="card-header">
            <h3 class="text-lg font-medium text-gray-900">Tool Details</h3>
          </div>
          <div class="card-body">
            <div v-if="!selectedTool" class="text-center py-8 text-gray-500">
              Select a tool to view details and execute
            </div>

            <div v-else class="space-y-6">
              <div>
                <h4 class="text-xl font-bold text-gray-900">{{ selectedTool.name }}</h4>
                <p class="text-gray-600 mt-1">{{ selectedTool.description }}</p>
              </div>

              <!-- Parameters Form -->
              <div v-if="Object.keys(toolParams).length > 0">
                <h5 class="font-medium text-gray-900 mb-3">Parameters</h5>
                <div class="space-y-4">
                  <div v-for="(_, key) in toolParams" :key="key">
                    <label :for="`param-${key}`" class="label">
                      {{ key }}
                      <span v-if="isRequired(selectedTool.input_schema, key)" class="text-red-500">*</span>
                      <span class="text-xs text-gray-400 ml-2">
                        ({{ getPropertyType(selectedTool.input_schema, key) }})
                      </span>
                    </label>
                    <input
                      :id="`param-${key}`"
                      v-model="toolParams[key]"
                      type="text"
                      class="input"
                      :placeholder="`Enter ${key}...`"
                    />
                  </div>
                </div>
              </div>

              <button
                @click="executeTool"
                class="btn btn-primary"
                :disabled="executing"
              >
                {{ executing ? 'Executing...' : 'Execute Tool' }}
              </button>
            </div>
          </div>
        </div>

        <!-- Execution Result -->
        <div v-if="executionResult !== null" class="card">
          <div class="card-header">
            <h3 class="text-lg font-medium text-gray-900">Execution Result</h3>
          </div>
          <div class="card-body">
            <pre class="bg-gray-50 p-4 rounded-lg overflow-x-auto text-sm font-mono">{{ JSON.stringify(executionResult, null, 2) }}</pre>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
