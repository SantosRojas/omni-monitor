import type { LoginResponse, UserResponse } from '../types'
import { apiGet, apiPost } from './client'

export function login(username: string, password: string): Promise<LoginResponse> {
  return apiPost<LoginResponse>('/auth/login', { username, password })
}

export function getMe(): Promise<UserResponse> {
  return apiGet<UserResponse>('/auth/me')
}

export function logout(): Promise<void> {
  return apiPost<void>('/auth/logout')
}
