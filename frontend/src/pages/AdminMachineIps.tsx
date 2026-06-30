import { useEffect, useMemo, useState } from 'react'
import {
  useReactTable,
  getCoreRowModel,
  createColumnHelper,
} from '@tanstack/react-table'
import { Plus, Pencil, Trash2 } from 'lucide-react'
import type { MachineIpWithSerial, Machine } from '../types'
import * as machinesApi from '../api/machines'
import { Spinner } from '../components/ui/Spinner'
import { Modal } from '../components/ui/Modal'
import { Badge } from '../components/ui/Badge'

const helper = createColumnHelper<MachineIpWithSerial>()

const hideSm = (id: string) =>
  ['id', 'port', 'label'].includes(id) ? 'hidden md:table-cell' : ''

export function AdminMachineIps() {
  const [data, setData] = useState<MachineIpWithSerial[]>([])
  const [machines, setMachines] = useState<Machine[]>([])
  const [loading, setLoading] = useState(true)
  const [modalOpen, setModalOpen] = useState(false)
  const [editing, setEditing] = useState<MachineIpWithSerial | null>(null)

  const [formMachineId, setFormMachineId] = useState(0)
  const [formIp, setFormIp] = useState('')
  const [formPort, setFormPort] = useState(9001)
  const [formLabel, setFormLabel] = useState('')

  const fetchData = () => {
    setLoading(true)
    Promise.all([
      machinesApi.listMachineIps(),
      machinesApi.listMachines(),
    ]).then(([ips, machs]) => {
      setData(ips)
      setMachines(machs)
    }).catch(console.error)
      .finally(() => setLoading(false))
  }

  useEffect(fetchData, [])

  const openCreate = () => {
    setEditing(null)
    setFormMachineId(machines[0]?.id || 0)
    setFormIp('')
    setFormPort(9001)
    setFormLabel('')
    setModalOpen(true)
  }

  const openEdit = (item: MachineIpWithSerial) => {
    setEditing(item)
    setFormMachineId(item.machine_id)
    setFormIp(item.ip_address)
    setFormPort(item.port ?? 9001)
    setFormLabel(item.label ?? '')
    setModalOpen(true)
  }

  const handleSave = async () => {
    try {
      if (editing) {
        await machinesApi.updateMachineIp(editing.id, {
          ip_address: formIp,
          port: formPort,
          label: formLabel,
        })
      } else {
        await machinesApi.createMachineIp({
          machine_id: formMachineId,
          ip_address: formIp,
          port: formPort,
          label: formLabel,
        })
      }
      setModalOpen(false)
      fetchData()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Error al guardar')
    }
  }

  const handleDelete = async (id: number) => {
    if (!confirm('¿Eliminar esta IP?')) return
    try {
      await machinesApi.deleteMachineIp(id)
      fetchData()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Error al eliminar')
    }
  }

  const columns = useMemo(() => [
    helper.accessor('id', { header: 'ID' }),
    helper.accessor('serial_number', { header: 'Serial' }),
    helper.accessor('ip_address', { header: 'Dirección IP' }),
    helper.accessor('port', { header: 'Puerto' }),
    helper.accessor('label', { header: 'Etiqueta' }),
    helper.accessor('is_active', {
      header: 'Activo',
      cell: i => <Badge variant={i.getValue() ? 'active' : 'inactive'}>{i.getValue() ? 'Sí' : 'No'}</Badge>,
    }),
    helper.display({
      id: 'actions',
      header: '',
      cell: ({ row }) => (
        <div className="flex gap-1">
          <button onClick={() => openEdit(row.original)} className="p-1.5 rounded-sm hover:bg-[var(--surface-hover)] cursor-pointer text-(--text-secondary)">
            <Pencil className="w-4 h-4" />
          </button>
          <button onClick={() => handleDelete(row.original.id)} className="p-1.5 rounded-sm hover:bg-[var(--surface-hover)] cursor-pointer text-[var(--danger)]">
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
  })

  return (
    <div>
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 mb-5">
        <h2 className="text-lg md:text-xl font-bold text-(--text-primary)">IPs de Máquinas</h2>
        <button onClick={openCreate} className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-sm bg-[var(--accent)] text-white hover:opacity-90 cursor-pointer">
          <Plus className="w-4 h-4" /> Nueva IP
        </button>
      </div>

      {loading ? <Spinner message="Cargando..." /> : (
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
          {data.length === 0 && <div className="text-center py-10 text-(--text-muted) text-sm">No hay IPs registradas</div>}
        </div>
      )}

      <Modal open={modalOpen} onClose={() => setModalOpen(false)} title={editing ? 'Editar IP' : 'Nueva IP'}>
        <div className="flex flex-col gap-4">
          {!editing && (
            <div>
              <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Máquina</label>
              <select value={formMachineId} onChange={e => setFormMachineId(Number(e.target.value))}
                className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none">
                {machines.map(m => <option key={m.id} value={m.id}>{m.serial_number}</option>)}
              </select>
            </div>
          )}
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Dirección IP</label>
            <input value={formIp} onChange={e => setFormIp(e.target.value)}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Puerto</label>
            <input type="number" value={formPort} onChange={e => setFormPort(Number(e.target.value))}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Etiqueta</label>
            <input value={formLabel} onChange={e => setFormLabel(e.target.value)}
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
    </div>
  )
}
