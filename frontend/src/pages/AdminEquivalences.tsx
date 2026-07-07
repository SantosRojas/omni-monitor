import { useEffect, useMemo, useState } from 'react'
import {
  useReactTable,
  getCoreRowModel,
  getFilteredRowModel,
  createColumnHelper,
} from '@tanstack/react-table'
import type { ColumnFiltersState } from '@tanstack/react-table'
import { Plus, Pencil, Trash2, Search } from 'lucide-react'
import type { Equivalence } from '../types'
import * as equivalencesApi from '../api/equivalences'
import { Spinner, Modal, ColumnFilter } from '../components/ui'

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
          <button onClick={() => openEdit(row.original)} className="p-1.5 rounded-sm hover:bg-[var(--surface-hover)] cursor-pointer text-(--text-secondary)">
            <Pencil className="w-4 h-4" />
          </button>
          <button onClick={() => openDelete(row.original)} className="p-1.5 rounded-sm hover:bg-[var(--surface-hover)] cursor-pointer text-[var(--danger)]">
            <Trash2 className="w-4 h-4" />
          </button>
        </div>
      ),
    }),
  ], [data])

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
    onColumnFiltersChange: setColumnFilters,
    onGlobalFilterChange: setGlobalFilter,
    globalFilterFn: 'includesString',
    state: { columnFilters, globalFilter },
  })

  return (
    <div>
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 mb-5">
        <h2 className="text-lg md:text-xl font-bold text-(--text-primary)">Equivalencias</h2>
        <button onClick={openCreate} className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-sm bg-[var(--accent)] text-white hover:opacity-90 cursor-pointer">
          <Plus className="w-4 h-4" /> Nueva Equivalencia
        </button>
      </div>

      <div className="relative mb-4">
        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-(--text-muted)" />
        <input
          value={globalFilter ?? ''}
          onChange={e => setGlobalFilter(e.target.value)}
          placeholder="Buscar en toda la tabla..."
          className="w-full pl-9 pr-3 py-2 text-sm border border-(--glass-border) rounded-sm bg-(--surface-btn) text-(--text-primary) outline-none focus:border-[var(--accent)]"
        />
      </div>

      {loading ? <Spinner message="Cargando equivalencias..." /> : (
        <div className="glass overflow-x-auto">
          <table className="w-full border-collapse">
            <thead>
              {table.getHeaderGroups().map(hg => (
                <tr key={hg.id}>
                  {hg.headers.map(h => (
                    <th key={h.id} className={`text-left px-4 py-3 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-[var(--border-subtle)] ${hideSm(h.id)}`}>
                      <div className="flex flex-col">
                        {h.column.columnDef.header as string}
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
        </div>
      )}

      {error && <div className="mb-4 p-3 rounded-sm bg-red-50 border border-red-200 text-red-700 text-sm">{error}</div>}

      <Modal open={modalOpen} onClose={() => setModalOpen(false)} title={editing ? 'Editar Equivalencia' : 'Nueva Equivalencia'}>
        <div className="flex flex-col gap-4">
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Nombre Interno (internal_name)</label>
            <input value={formInternalName} onChange={e => setFormInternalName(e.target.value)} disabled={!!editing}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Valor Numérico</label>
            <input type="number" value={formNumericValue} onChange={e => setFormNumericValue(e.target.value)} disabled={!!editing}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Nombre Display</label>
            <input value={formDisplayName} onChange={e => setFormDisplayName(e.target.value)}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div className="flex justify-end gap-2 mt-2">
            <button onClick={() => setModalOpen(false)}
              className="px-4 py-2 text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) cursor-pointer">Cancelar</button>
            <button onClick={handleSave}
              className="px-4 py-2 text-sm rounded-sm bg-[var(--accent)] text-white hover:opacity-90 cursor-pointer">Guardar</button>
          </div>
        </div>
      </Modal>

      <Modal open={deleteModalOpen} onClose={() => setDeleteModalOpen(false)} title="Eliminar Equivalencia">
        <div className="flex flex-col gap-4">
          <p className="text-sm text-(--text-secondary)">
            ¿Eliminar "{deleting?.display_name}" ({deleting?.internal_name} = {deleting?.numeric_value})?
          </p>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Motivo (opcional)</label>
            <input value={deleteReason} onChange={e => setDeleteReason(e.target.value)}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div className="flex justify-end gap-2 mt-2">
            <button onClick={() => setDeleteModalOpen(false)}
              className="px-4 py-2 text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) cursor-pointer">Cancelar</button>
            <button onClick={handleDelete}
              className="px-4 py-2 text-sm rounded-sm bg-[var(--danger)] text-white hover:opacity-90 cursor-pointer">Eliminar</button>
          </div>
        </div>
      </Modal>
    </div>
  )
}
