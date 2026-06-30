import type { ActiveDevice, PaginatedResponse, Patient, PatientDashboard, TelemetryReading, TherapyWithMachine } from '../types'
import { apiGet } from './client'

export function listPatients(page = 1, perPage = 20, search?: string): Promise<PaginatedResponse<Patient>> {
  let path = `/patients?page=${page}&per_page=${perPage}`
  if (search) path += `&search=${encodeURIComponent(search)}`
  return apiGet<PaginatedResponse<Patient>>(path)
}

export function getPatient(id: number): Promise<Patient> {
  return apiGet<Patient>(`/patients/${id}`)
}

export function getTherapies(patientId: number): Promise<TherapyWithMachine[]> {
  return apiGet<TherapyWithMachine[]>(`/patients/${patientId}/therapies`)
}

export function getHistory(patientId: number, page = 1, perPage = 50): Promise<PaginatedResponse<TelemetryReading>> {
  return apiGet<PaginatedResponse<TelemetryReading>>(`/patients/${patientId}/history?page=${page}&per_page=${perPage}`)
}

export function getActiveDevice(patientId: number): Promise<ActiveDevice> {
  return apiGet<ActiveDevice>(`/patients/${patientId}/active-device`)
}

export function getPatientDashboard(patientId: number, signalIds?: string, from?: string, to?: string): Promise<PatientDashboard> {
  const params = new URLSearchParams()
  if (signalIds) params.set('signal_ids', signalIds)
  if (from) params.set('from', from)
  if (to) params.set('to', to)
  const qs = params.toString()
  return apiGet<PatientDashboard>(`/patients/${patientId}/dashboard${qs ? '?' + qs : ''}`)
}

export function getTherapyDashboard(therapyId: number): Promise<PatientDashboard> {
  return apiGet<PatientDashboard>(`/therapies/${therapyId}/dashboard`)
}
