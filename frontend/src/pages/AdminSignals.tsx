import { useEffect, useMemo, useState } from 'react'
import {
  useReactTable,
  getCoreRowModel,
  getFilteredRowModel,
  getPaginationRowModel,
  getSortedRowModel,
  createColumnHelper,
} from '@tanstack/react-table'
import type { ColumnFiltersState, SortingState } from '@tanstack/react-table'
import { Pencil } from 'lucide-react'
import type { Signal } from '../types'
import * as signalsApi from '../api/signals'
import { Spinner, Modal, ColumnFilter, Button, Input, Label, Pagination } from '../components/ui'

const helper = createColumnHelper<Signal>()

const hideSm = (id: string) =>
  ['internal_name', 'unit'].includes(id) ? 'hidden md:table-cell' : ''

export function AdminSignals() {
  const [data, setData] = useState<Signal[]>([])
  const [loading, setLoading] = useState(true)
  const [selected, setSelected] = useState<Signal | null>(null)
  const [formDisplayName, setFormDisplayName] = useState('')
  const [formUnit, setFormUnit] = useState('')
  const [sorting, setSorting] = useState<SortingState>([])
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])

  const fetchData = () => {
    setLoading(true)
    signalsApi.listSignals()
      .then(setData)
      .catch(console.error)
      .finally(() => setLoading(false))
  }

  useEffect(fetchData, [])

  const openEdit = (signal: Signal) => {
    setSelected(signal)
    setFormDisplayName(signal.display_name ?? '')
    setFormUnit(signal.unit ?? '')
  }

  const handleSave = async () => {
    if (!selected) return
    try {
      await signalsApi.updateSignal(selected.id, {
        display_name: formDisplayName || undefined,
        unit: formUnit || undefined,
      })
      setData(prev =>
        prev.map(s =>
          s.id === selected.id
            ? { ...s, display_name: formDisplayName || undefined, unit: formUnit || undefined }
            : s,
        ),
      )
      setSelected(null)
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Error al guardar')
    }
  }

  const columns = useMemo(() => [
    helper.accessor('internal_name', { header: 'Nombre Interno' }),
    helper.accessor('display_name', {
      header: 'Nombre Display',
      cell: i => i.getValue() || <span className="text-(--text-muted) italic">—</span>,
    }),
    helper.accessor('unit', {
      header: 'Unidad',
      cell: i => i.getValue() || <span className="text-(--text-muted) italic">—</span>,
    }),
    helper.display({
      id: 'actions',
      header: '',
      cell: ({ row }) => (
        <div className="flex gap-1">
          <Button variant="icon" size="sm" onClick={() => openEdit(row.original)}>
            <Pencil className="w-4 h-4" />
          </Button>
        </div>
      ),
    }),
  ], [data])

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
    initialState: { pagination: { pageSize: 25 } },
  })

  return (
    <div>
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 mb-5">
        <h2 className="text-lg md:text-xl font-bold text-(--text-primary)">Señales</h2>
      </div>

      {loading ? <Spinner message="Cargando señales..." /> : (
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
          {data.length === 0 && <div className="text-center py-10 text-(--text-muted) text-sm">No hay señales</div>}
          {data.length > 0 && <Pagination table={table} />}
        </div>
      )}

      <Modal open={selected !== null} onClose={() => setSelected(null)} title="Editar Señal">
        <div className="flex flex-col gap-4">
          <div>
            <Label>Nombre Interno</Label>
            <Input value={selected?.internal_name ?? ''} disabled />
          </div>
          <div>
            <Label>Nombre Display</Label>
            <Input value={formDisplayName} onChange={e => setFormDisplayName(e.target.value)} />
          </div>
          <div>
            <Label>Unidad</Label>
            <Input value={formUnit} onChange={e => setFormUnit(e.target.value)} placeholder="mmHg, mL/h, ..." />
          </div>
          <div className="flex justify-end gap-2 mt-2">
            <Button variant="secondary" size="md" onClick={() => setSelected(null)}>Cancelar</Button>
            <Button variant="primary" size="md" onClick={handleSave}>Guardar</Button>
          </div>
        </div>
      </Modal>
    </div>
  )
}
