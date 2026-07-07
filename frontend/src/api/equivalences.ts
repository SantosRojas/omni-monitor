import type { Equivalence, CreateEquivalenceRequest, UpdateEquivalenceRequest } from '../types'
import { apiDelete, apiGet, apiPost, apiPut } from './client'

export function listEquivalences(): Promise<Equivalence[]> {
  return apiGet<Equivalence[]>('/admin/equivalences')
}

export function createEquivalence(req: CreateEquivalenceRequest): Promise<void> {
  return apiPost<void>('/admin/equivalences', req)
}

export function updateEquivalence(req: UpdateEquivalenceRequest): Promise<void> {
  return apiPut<void>('/admin/equivalences', req)
}

export function deleteEquivalence(signalId: number, numericValue: number, deletionReason?: string): Promise<void> {
  const params = deletionReason ? `?deletion_reason=${encodeURIComponent(deletionReason)}` : ''
  return apiDelete<void>(`/admin/equivalences/${signalId}/${numericValue}${params}`)
}
