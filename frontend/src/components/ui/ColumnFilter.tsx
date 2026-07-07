import type { Column } from '@tanstack/react-table'
import { Input } from './Input'

export function ColumnFilter({ column }: { column: Column<any, unknown> }) {
  return (
    <Input
      variant="column-filter"
      value={(column.getFilterValue() as string) ?? ''}
      onChange={e => column.setFilterValue(e.target.value)}
      placeholder="Filtrar..."
      onClick={e => e.stopPropagation()}
    />
  )
}
