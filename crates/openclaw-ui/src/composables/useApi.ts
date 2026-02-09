import { ref } from 'vue'
import { useAuthStore } from '@/stores/auth'

export interface RpcRequest {
  jsonrpc: '2.0'
  method: string
  params: Record<string, unknown>
  id: string
}

export interface RpcResponse<T = unknown> {
  jsonrpc: '2.0'
  result?: T
  error?: { code: number; message: string }
  id: string
}

export function useApi() {
  const authStore = useAuthStore()
  const loading = ref(false)
  const error = ref<string | null>(null)

  // Determine API base URL
  // In production (embedded), use the gateway API port
  // In development, use the Vite proxy
  const apiBaseUrl = import.meta.env.VITE_API_BASE_URL ||
    (import.meta.env.PROD
      ? `${window.location.protocol}//${window.location.hostname}:18789`
      : '')

  async function call<T>(method: string, params: Record<string, unknown> = {}): Promise<T> {
    loading.value = true
    error.value = null

    const request: RpcRequest = {
      jsonrpc: '2.0',
      method,
      params,
      id: crypto.randomUUID(),
    }

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
    }

    if (authStore.token) {
      headers['Authorization'] = `Bearer ${authStore.token}`
    }

    try {
      const response = await fetch(`${apiBaseUrl}/rpc`, {
        method: 'POST',
        headers,
        body: JSON.stringify(request),
      })

      const data: RpcResponse<T> = await response.json()

      if (data.error) {
        throw new Error(data.error.message)
      }

      return data.result as T
    } catch (e) {
      error.value = e instanceof Error ? e.message : 'Unknown error'
      throw e
    } finally {
      loading.value = false
    }
  }

  return { call, loading, error, apiBaseUrl }
}
