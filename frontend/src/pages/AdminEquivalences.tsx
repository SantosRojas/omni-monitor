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
import { Plus, Pencil, Trash2 } from 'lucide-react'
import type { Equivalence } from '../types'
import * as equivalencesApi from '../api/equivalences'
import { Modal, Button, Input, Label } from '../components/ui'
import { DataTable } from '../components/data-table'
import { useToast } from '../contexts/ToastContext'

const helper = createColumnHelper<Equivalence>()

const hideSm = (id: string) =>
  ['internal_name'].includes(id) ? 'hidden md:table-cell' : ''

export function AdminEquivalences() {
  const { showToast } = useToast()
  const [data, setData] = useState<Equivalence[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState<string | null>(null)
  const [modalOpen, setModalOpen] = useState(false)
  const [deleteModalOpen, setDeleteModalOpen] = useState(false)
  const [editing, setEditing] = useState<Equivalence | null>(null)
  const [deleting, setDeleting] = useState<Equivalence | null>(null)
  const [deleteReason, setDeleteReason] = useState('')

  const [formInternalName, setFormInternalName] = useState('')
  const [formNumericValue, setFormNumericValue] = useState('')
  const [formDisplayName, setFormDisplayName] = useState('')
  const [sorting, setSorting] = useState<SortingState>([])
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
  const [globalFilter, setGlobalFilter] = useState('')

  const fetchData = () => {
    setLoading(true)
    setError(null)
    equivalencesApi.listEquivalences()
      .then(setData)
      .catch(e => setError(e instanceof Error ? e.message : 'Error al cargar equivalencias'))
      .finally(() => setLoading(false))
  }

  useEffect(fetchData, [])

  const openCreate = () => {
    setEditing(null)
    setFormInternalName('')
    setFormNumericValue('')
    setFormDisplayName('')
    setModalOpen(true)
  }

  const openEdit = (item: Equivalence) => {
    setEditing(item)
    setFormInternalName(item.internal_name)
    setFormNumericValue(String(item.numeric_value))
    setFormDisplayName(item.display_name)
    setModalOpen(true)
  }

  const handleSave = async () => {
    try {
      if (editing) {
        await equivalencesApi.updateEquivalence({
          signal_id: editing.signal_id,
          numeric_value: editing.numeric_value,
          display_name: formDisplayName,
        })
      } else {
        await equivalencesApi.createEquivalence({
          internal_name: formInternalName,
          numeric_value: Number(formNumericValue),
          display_name: formDisplayName,
        })
      }
      setModalOpen(false)
      fetchData()
    } catch (e) {
      showToast(e instanceof Error ? e.message : 'Error al guardar')
    }
  }

  const openDelete = (item: Equivalence) => {
    setDeleting(item)
    setDeleteReason('')
    setDeleteModalOpen(true)
  }

  const handleDelete = async () => {
    if (!deleting) return
    try {
      await equivalencesApi.deleteEquivalence(deleting.signal_id, deleting.numeric_value, deleteReason || undefined)
      setDeleteModalOpen(false)
      setDeleting(null)
      fetchData()
    } catch (e) {
      showToast(e instanceof Error ? e.message : 'Error al eliminar')
    }
  }

  const columns = useMemo(() => [
    helper.accessor('internal_name', { header: 'Señal' }),
    helper.accessor('numeric_value', { header: 'Valor Numérico' }),
    helper.accessor('display_name', { header: 'Nombre Display' }),
    helper.display({
      id: 'actions',
      header: '',
      cell: ({ row }) => (
        <div className="flex gap-1">
          <Button variant="icon" size="sm" onClick={() => openEdit(row.original)}>
            <Pencil className="w-4 h-4" />
          </Button>
          <Button variant="icon" size="sm" onClick={() => openDelete(row.original)} className="!text-[var(--danger)]">
            <Trash2 className="w-4 h-4" />
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
    onGlobalFilterChange: setGlobalFilter,
    globalFilterFn: 'includesString',
    defaultColumn: { filterFn: 'includesString' },
    state: { sorting, columnFilters, globalFilter },
    initialState: { pagination: { pageSize: 25 } },
  })

  return (
    <div>
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 mb-5">
        <h2 className="text-lg md:text-xl font-bold text-(--text-primary)">Equivalencias</h2>
        <Button variant="primary" size="sm" icon={<Plus className="w-4 h-4" />} onClick={openCreate}>
          Nueva Equivalencia
        </Button>
      </div>

      <DataTable table={table} loading={loading}>
        <DataTable.Search />
        <DataTable.Grid
          emptyMessage="No hay equivalencias"
          hideSm={hideSm}
        />
        <DataTable.Pagination />
      </DataTable>

      {error && <div className="mb-4 p-3 rounded-sm bg-red-50 border border-red-200 text-red-700 text-sm">{error}</div>}

      <Modal open={modalOpen} onClose={() => setModalOpen(false)} title={editing ? 'Editar Equivalencia' : 'Nueva Equivalencia'}>
        <div className="flex flex-col gap-4">
          <div>
            <Label>Nombre Interno (internal_name)</Label>
            <Input value={formInternalName} onChange={e => setFormInternalName(e.target.value)} disabled={!!editing} />
          </div>
          <div>
            <Label>Valor Numérico</Label>
            <Input type="number" value={formNumericValue} onChange={e => setFormNumericValue(e.target.value)} disabled={!!editing} />
          </div>
          <div>
            <Label>Nombre Display</Label>
            <Input value={formDisplayName} onChange={e => setFormDisplayName(e.target.value)} />
          </div>
          <div className="flex justify-end gap-2 mt-2">
            <Button variant="secondary" size="md" onClick={() => setModalOpen(false)}>Cancelar</Button>
            <Button variant="primary" size="md" onClick={handleSave}>Guardar</Button>
          </div>
        </div>
      </Modal>

      <Modal open={deleteModalOpen} onClose={() => setDeleteModalOpen(false)} title="Eliminar Equivalencia">
        <div className="flex flex-col gap-4">
          <p className="text-sm text-(--text-secondary)">
            ¿Eliminar "{deleting?.display_name}" ({deleting?.internal_name} = {deleting?.numeric_value})?
          </p>
          <div>
            <Label>Motivo (opcional)</Label>
            <Input value={deleteReason} onChange={e => setDeleteReason(e.target.value)} />
          </div>
          <div className="flex justify-end gap-2 mt-2">
            <Button variant="secondary" size="md" onClick={() => setDeleteModalOpen(false)}>Cancelar</Button>
            <Button variant="danger" size="md" onClick={handleDelete}>Eliminar</Button>
          </div>
        </div>
      </Modal>
    </div>
  )
}
