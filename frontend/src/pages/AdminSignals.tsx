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
import { Modal, Button, Input, Label } from '../components/ui'
import { DataTable } from '../components/data-table'
import { useToast } from '../contexts/ToastContext'

const helper = createColumnHelper<Signal>()

const hideSm = (id: string) =>
  ['internal_name', 'unit'].includes(id) ? 'hidden md:table-cell' : ''

export function AdminSignals() {
  const { showToast } = useToast()
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
      .catch(e => showToast(e instanceof Error ? e.message : 'Error al cargar señales'))
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
      showToast(e instanceof Error ? e.message : 'Error al guardar')
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

      <DataTable table={table} loading={loading}>
        <DataTable.Grid
          emptyMessage="No hay señales"
          hideSm={hideSm}
        />
        <DataTable.Pagination />
      </DataTable>

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
