const LOCALE = 'es-PE'

export function formatDate(iso: string | null | undefined): string {
  if (!iso) return '-'
  return new Date(iso + 'Z').toLocaleString(LOCALE)
}

export function formatDateShort(iso: string | null | undefined): string {
  if (!iso) return '-'
  return new Date(iso + 'Z').toLocaleDateString(LOCALE)
}


