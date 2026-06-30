import { PackageOpen } from 'lucide-react'

export function EmptyState({ message }: { message?: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-10 text-(--text-muted)">
      <PackageOpen className="w-10 h-10 mb-2 opacity-50" />
      <p className="text-sm">{message || 'Sin datos'}</p>
    </div>
  )
}
