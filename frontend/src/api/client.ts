import { parseApiError } from './errors'

const API_BASE = '/api'

async function request<T>(
  method: string,
  path: string,
  body?: unknown,
  token?: string | null
): Promise<T> {
  const url = `${API_BASE}${path}`
  const headers: Record<string, string> = {
    'Content-Type': 'application/json',
  }
  if (token) {
    headers['Authorization'] = `Bearer ${token}`
  }

  const res = await fetch(url, {
    method,
    headers,
    body: body ? JSON.stringify(body) : undefined,
  })

  if (!res.ok) {
    throw await parseApiError(res)
  }

  if (res.status === 204) return undefined as T

  return res.json()
}

function getToken(): string | null {
  return localStorage.getItem('monitor_token')
}

export function apiGet<T>(path: string, token?: string | null): Promise<T> {
  return request<T>('GET', path, undefined, token ?? getToken())
}

export function apiPost<T>(path: string, body?: unknown, token?: string | null): Promise<T> {
  return request<T>('POST', path, body, token ?? getToken())
}

export function apiPut<T>(path: string, body?: unknown, token?: string | null): Promise<T> {
  return request<T>('PUT', path, body, token ?? getToken())
}

export function apiDelete<T>(path: string, token?: string | null): Promise<T> {
  return request<T>('DELETE', path, undefined, token ?? getToken())
}
