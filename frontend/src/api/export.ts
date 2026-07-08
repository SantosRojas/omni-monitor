import { parseApiError } from './errors'

const API_BASE = '/api'

async function fetchExport(patientIdOrTherapyId: number, type: 'patient' | 'therapy') {
  const token = localStorage.getItem('monitor_token')
  if (!token) return

  const path = type === 'patient'
    ? `${API_BASE}/patients/${patientIdOrTherapyId}/export`
    : `${API_BASE}/therapies/${patientIdOrTherapyId}/export`

  const res = await fetch(path, {
    headers: { Authorization: `Bearer ${token}` },
  })

  if (!res.ok) {
    throw await parseApiError(res)
  }

  const blob = await res.blob()
  const url = URL.createObjectURL(blob)
  const filename = type === 'patient'
    ? `patient_${patientIdOrTherapyId}_history.xlsx`
    : `therapy_${patientIdOrTherapyId}_data.xlsx`
  const a = document.createElement('a')
  a.href = url
  a.download = filename
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

export function triggerPatientExport(patientId: number) {
  return fetchExport(patientId, 'patient')
}

export function triggerTherapyExport(therapyId: number) {
  return fetchExport(therapyId, 'therapy')
}
