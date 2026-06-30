const API_BASE = '/api'

export async function triggerPatientExport(patientId: number) {
  const token = localStorage.getItem('monitor_token')
  if (!token) return

  const res = await fetch(`${API_BASE}/patients/${patientId}/export`, {
    headers: { Authorization: `Bearer ${token}` },
  })

  if (!res.ok) {
    const text = await res.text().catch(() => 'Error al exportar')
    throw new Error(`HTTP ${res.status}: ${text}`)
  }

  const blob = await res.blob()
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `patient_${patientId}_history.xlsx`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}

export async function triggerTherapyExport(therapyId: number) {
  const token = localStorage.getItem('monitor_token')
  if (!token) return

  const res = await fetch(`${API_BASE}/therapies/${therapyId}/export`, {
    headers: { Authorization: `Bearer ${token}` },
  })

  if (!res.ok) {
    const text = await res.text().catch(() => 'Error al exportar')
    throw new Error(`HTTP ${res.status}: ${text}`)
  }

  const blob = await res.blob()
  const url = URL.createObjectURL(blob)
  const a = document.createElement('a')
  a.href = url
  a.download = `therapy_${therapyId}_data.xlsx`
  document.body.appendChild(a)
  a.click()
  document.body.removeChild(a)
  URL.revokeObjectURL(url)
}
