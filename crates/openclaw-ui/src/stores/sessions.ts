import { defineStore } from 'pinia'
import { ref } from 'vue'
import { useApi } from '@/composables/useApi'

export interface SessionMessage {
  type: 'inbound' | 'outbound' | 'tool'
  content: string
  timestamp?: string
  tool_name?: string
}

export interface Session {
  session_key: string
  agent_id: string
  channel: string
  peer_id: string
  state: 'active' | 'paused' | 'ended'
  messages: SessionMessage[]
  message_count: number
  created_at: string
  last_activity: string
}

export interface SessionStats {
  total: number
  active: number
  by_channel: Record<string, number>
  by_agent: Record<string, number>
  total_messages: number
}

export const useSessionsStore = defineStore('sessions', () => {
  const sessions = ref<Session[]>([])
  const currentSession = ref<Session | null>(null)
  const stats = ref<SessionStats | null>(null)
  const loading = ref(false)

  async function fetchSessions(params: {
    limit?: number
    offset?: number
    channel?: string
    agent?: string
    state?: string
  } = {}): Promise<void> {
    loading.value = true
    try {
      const api = useApi()
      const result = await api.call<{ sessions: Session[]; total: number }>('session.list', params)
      sessions.value = result.sessions
    } finally {
      loading.value = false
    }
  }

  async function fetchSession(sessionKey: string): Promise<Session> {
    const api = useApi()
    const result = await api.call<Session>('session.history', { session_key: sessionKey })
    currentSession.value = result
    return result
  }

  async function fetchStats(): Promise<SessionStats> {
    const api = useApi()
    const result = await api.call<SessionStats>('session.stats')
    stats.value = result
    return result
  }

  async function createSession(agentId: string, channel: string = 'ui', peerId: string = 'web-user'): Promise<string> {
    const api = useApi()
    const result = await api.call<{ session_key: string }>('session.create', {
      agent_id: agentId,
      channel,
      peer_id: peerId,
    })
    return result.session_key
  }

  async function sendMessage(sessionKey: string, message: string, agentId: string = 'default'): Promise<string> {
    const api = useApi()
    const result = await api.call<{ response: string }>('session.message', {
      session_key: sessionKey,
      message,
      agent_id: agentId,
    })
    return result.response
  }

  async function endSession(sessionKey: string, reason: string = 'user_requested'): Promise<void> {
    const api = useApi()
    await api.call('session.end', { session_key: sessionKey, reason })
  }

  async function searchSessions(query: string, params: {
    channel?: string
    agent?: string
    limit?: number
  } = {}): Promise<Session[]> {
    const api = useApi()
    const result = await api.call<{ sessions: Session[] }>('session.search', { query, ...params })
    return result.sessions
  }

  return {
    sessions,
    currentSession,
    stats,
    loading,
    fetchSessions,
    fetchSession,
    fetchStats,
    createSession,
    sendMessage,
    endSession,
    searchSessions,
  }
})
