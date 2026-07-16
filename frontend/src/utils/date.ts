const LOCALE = 'es-PE'

function ensureZ(iso: string): string {
  return iso.endsWith('Z') ? iso : iso + 'Z'
}

export function formatDate(iso: string | null | undefined): string {
  if (!iso) return '-'
  return new Date(ensureZ(iso)).toLocaleString(LOCALE)
}

export function formatDateShort(iso: string | null | undefined): string {
  if (!iso) return '-'
  return new Date(ensureZ(iso)).toLocaleDateString(LOCALE)
}


