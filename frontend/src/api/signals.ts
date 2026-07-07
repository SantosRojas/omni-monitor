import type { Signal, UpdateSignalRequest } from '../types'
import { apiGet, apiPut } from './client'

export function listSignals(): Promise<Signal[]> {
  return apiGet<Signal[]>('/admin/signals')
}

export function updateSignal(id: number, req: UpdateSignalRequest): Promise<void> {
  return apiPut<void>(`/admin/signals/${id}`, req)
}
