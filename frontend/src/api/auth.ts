import type { LoginResponse, UserResponse } from '../types'
import { apiGet, apiPost } from './client'

export function login(username: string, password: string): Promise<LoginResponse> {
  return apiPost<LoginResponse>('/auth/login', { username, password }, null)
}

export function getMe(token: string): Promise<UserResponse> {
  return apiGet<UserResponse>('/auth/me', token)
}
