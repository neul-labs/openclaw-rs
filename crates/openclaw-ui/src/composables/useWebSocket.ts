import { ref, onMounted, onUnmounted } from 'vue'
import { useAuthStore } from '@/stores/auth'

export interface UiEvent {
  type: string
  [key: string]: unknown
}

export interface UiEventEnvelope {
  id: string
  timestamp: string
  event: UiEvent
}

export function useWebSocket() {
  const authStore = useAuthStore()
  const connected = ref(false)
  const events = ref<UiEventEnvelope[]>([])
  let ws: WebSocket | null = null
  let reconnectTimer: number | null = null

  // Determine WebSocket URL
  const wsBaseUrl = import.meta.env.VITE_WS_BASE_URL ||
    (import.meta.env.PROD
      ? `ws://${window.location.hostname}:18789`
      : `ws://${window.location.hostname}:18789`)

  function connect() {
    if (ws?.readyState === WebSocket.OPEN) return

    const url = new URL(`${wsBaseUrl}/ws`)
    if (authStore.token) {
      url.searchParams.set('token', authStore.token)
    }

    ws = new WebSocket(url.toString())

    ws.onopen = () => {
      connected.value = true
      // Subscribe to events
      ws?.send(JSON.stringify({
        jsonrpc: '2.0',
        method: 'events.subscribe',
        params: {},
        id: crypto.randomUUID(),
      }))
    }

    ws.onclose = () => {
      connected.value = false
      // Reconnect after delay
      reconnectTimer = window.setTimeout(connect, 3000)
    }

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        if (data.method === 'event' && data.params) {
          events.value.unshift(data.params)
          // Keep only last 100 events
          if (events.value.length > 100) {
            events.value.pop()
          }
        }
      } catch (e) {
        console.error('Failed to parse WebSocket message:', e)
      }
    }

    ws.onerror = (error) => {
      console.error('WebSocket error:', error)
    }
  }

  function disconnect() {
    if (reconnectTimer) {
      clearTimeout(reconnectTimer)
      reconnectTimer = null
    }
    ws?.close()
    ws = null
    connected.value = false
  }

  function send(method: string, params: Record<string, unknown> = {}): void {
    if (ws?.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        jsonrpc: '2.0',
        method,
        params,
        id: crypto.randomUUID(),
      }))
    }
  }

  function clearEvents() {
    events.value = []
  }

  onMounted(connect)
  onUnmounted(disconnect)

  return { connected, events, send, connect, disconnect, clearEvents }
}
