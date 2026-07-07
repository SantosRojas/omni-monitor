import { useEffect, useMemo, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import {
  useReactTable,
  getCoreRowModel,
  getPaginationRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  createColumnHelper,
} from '@tanstack/react-table'
import type { ColumnFiltersState, SortingState } from '@tanstack/react-table'
import { ArrowLeft, ChevronLeft, ChevronRight } from 'lucide-react'
import type { TelemetryReading } from '../types'
import * as patientsApi from '../api/patients'
import { Spinner, ColumnFilter } from '../components/ui'
import { formatDate } from '../utils/date'

const helper = createColumnHelper<TelemetryReading>()

const hideSm = (id: string) =>
  ['signal_id', 'unit'].includes(id) ? 'hidden md:table-cell' : ''

export function PatientHistory() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [data, setData] = useState<TelemetryReading[]>([])
  const [loading, setLoading] = useState(true)
  const [page, setPage] = useState(1)
  const [total, setTotal] = useState(0)
  const [sorting, setSorting] = useState<SortingState>([])
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
  const perPage = 50

  useEffect(() => {
    if (!id) return
    setLoading(true)
    patientsApi.getHistory(Number(id), page, perPage)
      .then(res => { setData(res.data); setTotal(res.total) })
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [id, page])

  const columns = useMemo(() => [
    helper.accessor('timestamp', {
      header: 'Fecha/Hora',
      cell: i => formatDate(i.getValue()),
    }),
    helper.accessor('signal_id', { header: 'Signal ID' }),
    helper.accessor('physical_value', { header: 'Valor' }),
    helper.accessor('unit', { header: 'Unidad' }),
  ], [])

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    state: { sorting, columnFilters },
    manualPagination: true,
    pageCount: Math.ceil(total / perPage),
  })

  return (
    <div>
      <button onClick={() => navigate(`/patients/${id}`)} className="flex items-center gap-1.5 text-sm text-(--text-secondary) hover:text-(--text-primary) mb-4 cursor-pointer">
        <ArrowLeft className="w-4 h-4" /> Volver al paciente
      </button>

      <h2 className="text-lg md:text-xl font-bold mb-5 text-(--text-primary)">Historial de Telemetría</h2>

      {loading ? <Spinner message="Cargando historial..." /> : (
        <div className="glass overflow-x-auto">
          <table className="w-full border-collapse">
            <thead>
              {table.getHeaderGroups().map(hg => (
                <tr key={hg.id}>
                  {hg.headers.map(h => (
                    <th key={h.id} onClick={h.column.getToggleSortingHandler()} className={`text-left px-4 py-3 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-[var(--border-subtle)] cursor-pointer select-none ${hideSm(h.id)}`}>
                      <div className="flex flex-col">
                        <div className="flex items-center gap-1">
                          {h.column.columnDef.header as string}
                          {h.column.getIsSorted() && <span className="text-[10px]">{h.column.getIsSorted() === 'asc' ? '▲' : '▼'}</span>}
                        </div>
                        {h.column.getCanFilter() && <ColumnFilter column={h.column} />}
                      </div>
                    </th>
                  ))}
                </tr>
              ))}
            </thead>
            <tbody>
              {table.getRowModel().rows.map(row => (
                <tr key={row.id} className="hover:bg-(--surface-row-hover) transition-colors">
                  {row.getVisibleCells().map(cell => (
                    <td key={cell.id} className={`px-4 py-3 text-sm text-(--text-secondary) border-b border-[var(--border-subtle)] ${hideSm(cell.column.id)}`}>
                      {cell.column.columnDef.cell ? (cell.column.columnDef.cell as any)(cell.getContext()) : cell.getValue() as string}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>

          {data.length === 0 && <div className="text-center py-10 text-(--text-muted) text-sm">Sin datos de telemetría</div>}
        </div>
      )}

      <div className="flex items-center justify-center gap-1 sm:gap-2 pt-4">
        <button onClick={() => setPage(p => Math.max(1, p - 1))} disabled={page <= 1}
          className="flex items-center gap-1 px-2 sm:px-3 py-1.5 text-xs sm:text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) disabled:opacity-30 cursor-pointer disabled:cursor-default">
          <ChevronLeft className="w-3.5 h-3.5" /> <span className="hidden sm:inline">Anterior</span>
        </button>
        <span className="text-xs sm:text-sm text-(--text-muted)">{page} / {Math.ceil(total / perPage)}</span>
        <button onClick={() => setPage(p => p + 1)} disabled={page >= Math.ceil(total / perPage)}
          className="flex items-center gap-1 px-2 sm:px-3 py-1.5 text-xs sm:text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) disabled:opacity-30 cursor-pointer disabled:cursor-default">
          <span className="hidden sm:inline">Siguiente</span> <ChevronRight className="w-3.5 h-3.5" />
        </button>
      </div>
    </div>
  )
}
