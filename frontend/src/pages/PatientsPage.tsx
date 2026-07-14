import { useMemo, useState, useEffect, useCallback } from 'react'
import { useNavigate } from 'react-router-dom'
import {
  useReactTable,
  getCoreRowModel,
  getSortedRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  createColumnHelper,
  type SortingState,
  type ColumnFiltersState,
} from '@tanstack/react-table'
import type { Patient } from '../types'
import * as patientsApi from '../api/patients'
import { Spinner, Badge, SearchInput, Button } from '../components/ui'
import { useToast } from '../contexts/ToastContext'
import { formatDateShort } from '../utils/date'
import { ActiveTherapiesTable } from '../components/ActiveTherapiesTable'

const columnHelper = createColumnHelper<Patient>()

const hideSm = (id: string) =>
  ['created_at'].includes(id) ? 'hidden md:table-cell' : ''

export function PatientsPage() {
  const { showToast } = useToast()
  const navigate = useNavigate()
  const [data, setData] = useState<Patient[]>([])
  const [loading, setLoading] = useState(true)
  const [search, setSearch] = useState('')
  const [sorting, setSorting] = useState<SortingState>([])
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
  const [pagination, setPagination] = useState({ pageIndex: 0, pageSize: 20 })
  const [total, setTotal] = useState(0)

  const fetchData = useCallback(async (page: number, perPage: number, q: string) => {
    setLoading(true)
    try {
      const res = await patientsApi.listPatients(page, perPage, q || undefined)
      setData(res.data)
      setTotal(res.total)
    } catch (e) {
      showToast(e instanceof Error ? e.message : 'Error al cargar pacientes')
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => {
    fetchData(pagination.pageIndex + 1, pagination.pageSize, search)
  }, [pagination.pageIndex, pagination.pageSize, search, fetchData])

  const columns = useMemo(() => [
    columnHelper.accessor('patient_id_str', {
      header: 'Paciente ID',
    }),
    columnHelper.accessor('created_at', {
      header: 'Creado',
      cell: info => formatDateShort(info.getValue()),
    }),
    columnHelper.accessor('completed_therapy_count', {
      header: 'Terapias Comp.',
      cell: info => (
        <Badge variant={info.getValue() && info.getValue()! > 0 ? 'active' : 'inactive'}>
          {info.getValue() ?? 0}
        </Badge>
      ),
    }),
  ], [])

  const table = useReactTable({
    data,
    columns,
    state: { sorting, columnFilters, pagination },
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    onPaginationChange: setPagination,
    getCoreRowModel: getCoreRowModel(),
    getSortedRowModel: getSortedRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    getPaginationRowModel: getPaginationRowModel(),
    manualPagination: true,
    pageCount: Math.ceil(total / pagination.pageSize),
    manualFiltering: true,
  })

  return (
    <div>
      <div className="flex flex-col sm:flex-row items-start sm:items-center gap-3 mb-5">
        <div className="w-full sm:max-w-md">
          <SearchInput
            value={search}
            onChange={e => { setSearch(e.target.value); setPagination(p => ({ ...p, pageIndex: 0 })) }}
            placeholder="Buscar paciente..."
          />
        </div>
      </div>

      <ActiveTherapiesTable />

      {loading ? <Spinner message="Cargando pacientes..." /> : (
        <>
          <div className="glass overflow-x-auto">
            <h2 className="text-sm font-semibold uppercase tracking-wider text-(--text-muted) px-4 py-3">
              Historial de pacientes
            </h2>
            <table className="w-full border-collapse">
              <thead>
                {table.getHeaderGroups().map(hg => (
                  <tr key={hg.id}>
                    {hg.headers.map(h => (
                      <th
                        key={h.id}
                        onClick={h.column.getToggleSortingHandler()}
                        className={`text-left px-4 py-3 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-[var(--border-subtle)] cursor-pointer select-none ${hideSm(h.id)}`}
                        style={{ width: h.getSize() }}
                      >
                        {h.column.columnDef.header as string}
                        {{ asc: ' ▲', desc: ' ▼' }[h.column.getIsSorted() as string] ?? ''}
                      </th>
                    ))}
                  </tr>
                ))}
              </thead>
              <tbody>
                {table.getRowModel().rows.map(row => (
                  <tr
                    key={row.id}
                    onClick={() => navigate(`/patients/${row.original.id}`)}
                    className="cursor-pointer hover:bg-(--surface-row-hover) transition-colors"
                  >
                    {row.getVisibleCells().map(cell => (
                      <td key={cell.id} className={`px-4 py-3 text-sm text-(--text-secondary) border-b border-[var(--border-subtle)] ${hideSm(cell.column.id)}`}>
                        {cell.column.columnDef.cell ? (cell.column.columnDef.cell as any)(cell.getContext()) : cell.getValue() as string}
                      </td>
                    ))}
                  </tr>
                ))}
              </tbody>
            </table>

            {data.length === 0 && !loading && (
              <div className="text-center py-10 text-(--text-muted) text-sm">No se encontraron pacientes</div>
            )}
          </div>
        </>
      )}

      <div className="flex items-center justify-center gap-1 sm:gap-2 pt-4">
        <Button variant="secondary" size="sm" onClick={() => table.previousPage()} disabled={!table.getCanPreviousPage()}>
          Anterior
        </Button>
        <span className="text-xs sm:text-sm text-(--text-muted) px-1">
          {pagination.pageIndex + 1} / {table.getPageCount()}
        </span>
        <Button variant="secondary" size="sm" onClick={() => table.nextPage()} disabled={!table.getCanNextPage()}>
          Siguiente
        </Button>
      </div>
    </div>
  )
}
