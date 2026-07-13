import type { TherapyComment } from '../types'
import { apiGet, apiPost, apiDelete } from './client'

export function getTherapyComments(therapyId: number): Promise<TherapyComment[]> {
  return apiGet<TherapyComment[]>(`/therapies/${therapyId}/comments`)
}

export function createTherapyComment(therapyId: number, comment: string): Promise<TherapyComment> {
  return apiPost<TherapyComment>(`/therapies/${therapyId}/comments`, { comment })
}

export function deleteTherapyComment(therapyId: number, commentId: number, reason: string): Promise<void> {
  return apiDelete<void>(`/therapies/${therapyId}/comments/${commentId}`, { deletion_reason: reason })
}
