import type { UserResponse, CreateUserRequest, UpdateUserRequest } from '../types'
import { apiDelete, apiGet, apiPost, apiPut } from './client'

export function listUsers(): Promise<UserResponse[]> {
  return apiGet<UserResponse[]>('/users')
}

export function createUser(req: CreateUserRequest): Promise<UserResponse> {
  return apiPost<UserResponse>('/users', req)
}

export function updateUser(id: number, req: UpdateUserRequest): Promise<UserResponse> {
  return apiPut<UserResponse>(`/users/${id}`, req)
}

export function deleteUser(id: number): Promise<void> {
  return apiDelete<void>(`/users/${id}`)
}
