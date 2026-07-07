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
import { Spinner, Modal, ColumnFilter, Button, Input, Label, SearchInput, Pagination } from '../components/ui'

const helper = createColumnHelper<Equivalence>()

const hideSm = (id: string) =>
  ['internal_name'].includes(id) ? 'hidden md:table-cell' : ''

export function AdminEquivalences() {
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
      alert(e instanceof Error ? e.message : 'Error al guardar')
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
      alert(e instanceof Error ? e.message : 'Error al eliminar')
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

      <div className="mb-4">
        <SearchInput value={globalFilter ?? ''} onChange={e => setGlobalFilter(e.target.value)} placeholder="Buscar en toda la tabla..." />
      </div>

      {loading ? <Spinner message="Cargando equivalencias..." /> : (
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
          {data.length === 0 && !error && <div className="text-center py-10 text-(--text-muted) text-sm">No hay equivalencias</div>}
          {data.length > 0 && <Pagination table={table} />}
        </div>
      )}

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
