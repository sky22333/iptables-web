const TOKEN_KEY = 'realm_web_token'

export interface TrafficTotals {
  tcp_rx: number
  tcp_tx: number
  udp_rx: number
  udp_tx: number
}

export interface TrafficSnapshot {
  rule_id: number
  local_port: number
  totals: TrafficTotals
  quota_bytes: number | null
  quota_used_ratio: number | null
}

export interface RuleRecord {
  id: number
  local_port: number
  listen_host: string
  target_host: string
  target_port: number
  enabled: boolean
  quota_bytes: number | null
  quota_period: string
  period_start: string | null
  created_at: string
  updated_at: string
}

export interface RuleWithTraffic {
  rule: RuleRecord
  traffic: TrafficSnapshot
}

export interface DashboardStats {
  rule_count: number
  active_count: number
  total_traffic_bytes: number
  quota_blocked_count: number
}

export interface ApiResponse<T> {
  success: boolean
  message?: string
  data?: T
}

export interface UpdateRulePayload {
  target_host?: string
  target_port?: number
  enabled?: boolean
  quota_gb?: number
  unset_quota?: boolean
  quota_period?: string
}

export function getToken(): string | null {
  return localStorage.getItem(TOKEN_KEY)
}

export function setToken(token: string) {
  localStorage.setItem(TOKEN_KEY, token)
}

export function clearToken() {
  localStorage.removeItem(TOKEN_KEY)
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const headers = new Headers(init?.headers)
  if (!headers.has('Content-Type') && init?.body) {
    headers.set('Content-Type', 'application/json')
  }
  const token = getToken()
  if (token) {
    headers.set('Authorization', `Bearer ${token}`)
  }

  const res = await fetch(`/api${path}`, { ...init, headers })

  if (res.status === 401) {
    clearToken()
    window.location.href = '/login'
    throw new Error('未登录或令牌已过期')
  }

  if (!res.ok) {
    let message = `请求失败 (${res.status})`
    try {
      const json = await res.json()
      if (json.message) message = json.message
    } catch {
      const text = await res.text()
      if (text) message = text
    }
    throw new Error(message)
  }

  if (res.status === 204) {
    return undefined as T
  }

  return res.json() as Promise<T>
}

export const api = {
  login(username: string, password: string) {
    return request<{ success: boolean; token?: string; message?: string }>('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ username, password }),
    })
  },

  stats() {
    return request<DashboardStats>('/stats')
  },

  listRules() {
    return request<RuleWithTraffic[]>('/rules')
  },

  addRules(body: {
    mode: string
    targets: string
    start_port?: number
    ports?: number[]
    quota_gb?: number
    quota_period?: string
  }) {
    return request<ApiResponse<RuleRecord[]>>('/rules', {
      method: 'POST',
      body: JSON.stringify(body),
    })
  },

  updateRule(port: number, body: UpdateRulePayload) {
    return request<ApiResponse<RuleRecord>>(`/rules/${port}`, {
      method: 'PATCH',
      body: JSON.stringify(body),
    })
  },

  toggleRule(port: number) {
    return request<ApiResponse<RuleRecord>>(`/rules/${port}/toggle`, { method: 'POST' })
  },

  resetTraffic(port: number) {
    return request<ApiResponse<void>>(`/rules/${port}/traffic/reset`, { method: 'POST' })
  },

  deleteRule(port: number) {
    return request<ApiResponse<void>>(`/rules/${port}`, { method: 'DELETE' })
  },

  deleteBatch(ports: number[]) {
    return request<ApiResponse<{ deleted: number[]; failed: number[] }>>('/rules/batch', {
      method: 'DELETE',
      body: JSON.stringify({ ports }),
    })
  },
}
