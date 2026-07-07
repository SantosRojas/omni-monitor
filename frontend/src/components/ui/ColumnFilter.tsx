import type { Column } from '@tanstack/react-table'

export function ColumnFilter({ column }: { column: Column<any, unknown> }) {
  return (
    <input
      value={(column.getFilterValue() as string) ?? ''}
      onChange={e => column.setFilterValue(e.target.value)}
      placeholder="Filtrar..."
      className="w-full px-2 py-1 text-xs border border-(--glass-border) rounded-sm bg-(--surface-btn) outline-none focus:border-[var(--accent)] mt-1.5"
      onClick={e => e.stopPropagation()}
    />
  )
}
