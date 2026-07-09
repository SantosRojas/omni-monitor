import { createContext, useContext, type ReactNode } from 'react'
import type { Table } from '@tanstack/react-table'
import { Spinner, SearchInput, Pagination, ColumnFilter } from '../ui'

interface DataTableContextValue {
  table: Table<any>
  loading: boolean
}

const DataTableContext = createContext<DataTableContextValue | null>(null)

function useDataTable(): DataTableContextValue {
  const ctx = useContext(DataTableContext)
  if (!ctx) {
    throw new Error('DataTable sub-components must be used within <DataTable>')
  }
  return ctx
}

interface DataTableProps {
  table: Table<any>
  loading?: boolean
  children: ReactNode
}

function DataTable({ table, loading = false, children }: DataTableProps) {
  return (
    <DataTableContext.Provider value={{ table, loading }}>
      {children}
    </DataTableContext.Provider>
  )
}

interface DataTableSearchProps {
  placeholder?: string
}

DataTable.Search = function Search({ placeholder }: DataTableSearchProps) {
  const { table } = useDataTable()
  const value = table.getState().globalFilter ?? ''

  return (
    <div className="mb-4">
      <SearchInput
        value={value}
        onChange={e => table.setGlobalFilter(e.target.value)}
        placeholder={placeholder ?? 'Buscar en toda la tabla...'}
      />
    </div>
  )
}

interface DataTableGridProps {
  emptyMessage?: string
  hideSm?: (columnId: string) => string
}

DataTable.Grid = function Grid({ emptyMessage, hideSm }: DataTableGridProps) {
  const { table, loading } = useDataTable()

  if (loading) {
    return <Spinner />
  }

  const rows = table.getRowModel().rows

  return (
    <div className="glass overflow-x-auto">
      <table className="w-full border-collapse">
        <thead>
          {table.getHeaderGroups().map(hg => (
            <tr key={hg.id}>
              {hg.headers.map(h => (
                <th
                  key={h.id}
                  onClick={h.column.getToggleSortingHandler()}
                  className={`text-left px-4 py-3 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-[var(--border-subtle)] cursor-pointer select-none ${hideSm?.(h.id) ?? ''}`}
                >
                  <div className="flex flex-col">
                    <div className="flex items-center gap-1">
                      {h.column.columnDef.header as string}
                      {h.column.getIsSorted() && (
                        <span className="text-[10px]">
                          {h.column.getIsSorted() === 'asc' ? '▲' : '▼'}
                        </span>
                      )}
                    </div>
                    {h.column.getCanFilter() && <ColumnFilter column={h.column} />}
                  </div>
                </th>
              ))}
            </tr>
          ))}
        </thead>
        <tbody>
          {rows.map(row => (
            <tr key={row.id} className="hover:bg-(--surface-row-hover) transition-colors">
              {row.getVisibleCells().map(cell => (
                <td
                  key={cell.id}
                  className={`px-4 py-3 text-sm text-(--text-secondary) border-b border-[var(--border-subtle)] ${hideSm?.(cell.column.id) ?? ''}`}
                >
                  {cell.column.columnDef.cell
                    ? (cell.column.columnDef.cell as any)(cell.getContext())
                    : (cell.getValue() as string)}
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
      {rows.length === 0 && emptyMessage && (
        <div className="text-center py-10 text-(--text-muted) text-sm">{emptyMessage}</div>
      )}
    </div>
  )
}

DataTable.Pagination = function PaginationWrapper() {
  const { table } = useDataTable()
  return <Pagination table={table} />
}

export { DataTable }
