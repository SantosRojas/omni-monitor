import { useEffect, useMemo, useState } from 'react'
import {
  useReactTable,
  getCoreRowModel,
  createColumnHelper,
} from '@tanstack/react-table'
import { Pencil } from 'lucide-react'
import type { Signal } from '../types'
import * as signalsApi from '../api/signals'
import { Spinner } from '../components/ui/Spinner'
import { Modal } from '../components/ui/Modal'

const helper = createColumnHelper<Signal>()

const hideSm = (id: string) =>
  ['internal_name', 'unit'].includes(id) ? 'hidden md:table-cell' : ''

export function AdminSignals() {
  const [data, setData] = useState<Signal[]>([])
  const [loading, setLoading] = useState(true)
  const [selected, setSelected] = useState<Signal | null>(null)
  const [formDisplayName, setFormDisplayName] = useState('')
  const [formUnit, setFormUnit] = useState('')

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
    helper.accessor('id', { header: 'ID' }),
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
          <button onClick={() => openEdit(row.original)} className="p-1.5 rounded-sm hover:bg-[var(--surface-hover)] cursor-pointer text-(--text-secondary)">
            <Pencil className="w-4 h-4" />
          </button>
        </div>
      ),
    }),
  ], [data])

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
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
                    <th key={h.id} className={`text-left px-4 py-3 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-[var(--border-subtle)] ${hideSm(h.id)}`}>
                      {h.column.columnDef.header as string}
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
        </div>
      )}

      <Modal open={selected !== null} onClose={() => setSelected(null)} title="Editar Señal">
        <div className="flex flex-col gap-4">
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Nombre Interno</label>
            <input value={selected?.internal_name ?? ''} disabled
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none" />
          </div>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Nombre Display</label>
            <input value={formDisplayName} onChange={e => setFormDisplayName(e.target.value)}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Unidad</label>
            <input value={formUnit} onChange={e => setFormUnit(e.target.value)} placeholder="mmHg, mL/h, ..."
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div className="flex justify-end gap-2 mt-2">
            <button onClick={() => setSelected(null)}
              className="px-4 py-2 text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) cursor-pointer">Cancelar</button>
            <button onClick={handleSave}
              className="px-4 py-2 text-sm rounded-sm bg-[var(--accent)] text-white hover:opacity-90 cursor-pointer">Guardar</button>
          </div>
        </div>
      </Modal>
    </div>
  )
}
