import { useEffect, useMemo, useState } from 'react'
import {
  useReactTable,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  createColumnHelper,
} from '@tanstack/react-table'
import type { ColumnFiltersState, SortingState } from '@tanstack/react-table'
import { Plus, Pencil, Trash2 } from 'lucide-react'
import type { MachineIpWithSerial, Machine } from '../types'
import * as machinesApi from '../api/machines'
import { Spinner, Modal, Badge, Select, ColumnFilter, Button, Input, Label, SearchInput } from '../components/ui'
import { useToast } from '../contexts/ToastContext'

const helper = createColumnHelper<MachineIpWithSerial>()

const hideSm = (id: string) =>
  ['port', 'label'].includes(id) ? 'hidden md:table-cell' : ''

export function AdminMachineIps() {
  const { showToast } = useToast()
  const [data, setData] = useState<MachineIpWithSerial[]>([])
  const [machines, setMachines] = useState<Machine[]>([])
  const [loading, setLoading] = useState(true)
  const [modalOpen, setModalOpen] = useState(false)
  const [editing, setEditing] = useState<MachineIpWithSerial | null>(null)

  const [formMachineId, setFormMachineId] = useState(0)
  const [formIp, setFormIp] = useState('')
  const [formPort, setFormPort] = useState(9001)
  const [formLabel, setFormLabel] = useState('')
  const [formIsActive, setFormIsActive] = useState(true)
  const [sorting, setSorting] = useState<SortingState>([])
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
  const [globalFilter, setGlobalFilter] = useState('')

  const fetchData = () => {
    setLoading(true)
    Promise.all([
      machinesApi.listMachineIps(),
      machinesApi.listMachines(),
    ]).then(([ips, machs]) => {
      setData(ips)
      setMachines(machs)
    }).catch(e => showToast(e instanceof Error ? e.message : 'Error al cargar'))
      .finally(() => setLoading(false))
  }

  useEffect(fetchData, [])

  const openCreate = () => {
    setEditing(null)
    setFormMachineId(machines[0]?.id || 0)
    setFormIp('')
    setFormPort(9001)
    setFormLabel('')
    setFormIsActive(true)
    setModalOpen(true)
  }

  const openEdit = (item: MachineIpWithSerial) => {
    setEditing(item)
    setFormMachineId(item.machine_id)
    setFormIp(item.ip_address)
    setFormPort(item.port ?? 9001)
    setFormLabel(item.label ?? '')
    setFormIsActive(item.is_active)
    setModalOpen(true)
  }

  const handleSave = async () => {
    try {
      if (editing) {
        await machinesApi.updateMachineIp(editing.id, {
          ip_address: formIp,
          port: formPort,
          label: formLabel,
          is_active: formIsActive,
        })
      } else {
        await machinesApi.createMachineIp({
          machine_id: formMachineId,
          ip_address: formIp,
          port: formPort,
          label: formLabel,
          is_active: formIsActive,
        })
      }
      setModalOpen(false)
      fetchData()
    } catch (e) {
      showToast(e instanceof Error ? e.message : 'Error al guardar')
    }
  }

  const handleDelete = async (id: number) => {
    if (!confirm('¿Eliminar esta IP?')) return
    try {
      await machinesApi.deleteMachineIp(id)
      fetchData()
    } catch (e) {
      showToast(e instanceof Error ? e.message : 'Error al eliminar')
    }
  }

  const columns = useMemo(() => [
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
          <Button variant="icon" size="sm" onClick={() => openEdit(row.original)}>
            <Pencil className="w-4 h-4" />
          </Button>
          <Button variant="icon" size="sm" onClick={() => handleDelete(row.original.id)} className="!text-[var(--danger)]">
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
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    onGlobalFilterChange: setGlobalFilter,
    globalFilterFn: 'includesString',
    state: { sorting, columnFilters, globalFilter },
  })

  return (
    <div>
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 mb-5">
        <h2 className="text-lg md:text-xl font-bold text-(--text-primary)">IPs de Máquinas</h2>
        <Button variant="primary" size="sm" icon={<Plus className="w-4 h-4" />} onClick={openCreate}>
          Nueva IP
        </Button>
      </div>

      <div className="mb-4">
        <SearchInput value={globalFilter ?? ''} onChange={e => setGlobalFilter(e.target.value)} placeholder="Buscar en toda la tabla..." />
      </div>

      {loading ? <Spinner message="Cargando..." /> : (
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
          {data.length === 0 && <div className="text-center py-10 text-(--text-muted) text-sm">No hay IPs registradas</div>}
        </div>
      )}

      <Modal open={modalOpen} onClose={() => setModalOpen(false)} title={editing ? 'Editar IP' : 'Nueva IP'}>
        <div className="flex flex-col gap-4">
          {!editing && (
            <div>
              <Label>Máquina</Label>
              <Select
                options={machines.map(m => ({ value: m.id, label: m.serial_number }))}
                value={formMachineId}
                onChange={setFormMachineId}
                placeholder="Seleccionar máquina…"
              />
            </div>
          )}
          <div>
            <Label>Dirección IP</Label>
            <Input value={formIp} onChange={e => setFormIp(e.target.value)} />
          </div>
          <div>
            <Label>Puerto</Label>
            <Input type="number" value={formPort} onChange={e => setFormPort(Number(e.target.value))} />
          </div>
          <div className="flex items-center gap-3">
            <Label className="!mb-0 cursor-pointer" onClick={() => setFormIsActive(!formIsActive)}>Activo</Label>
            <button
              type="button"
              onClick={() => setFormIsActive(!formIsActive)}
              className={`relative inline-flex h-5 w-9 shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 focus:outline-none ${formIsActive ? 'bg-(--accent)' : 'bg-(--surface-btn-hover)'}`}
            >
              <span className={`inline-block h-4 w-4 translate-y-0 transform rounded-full bg-white shadow-sm ring-0 transition-transform duration-200 ${formIsActive ? 'translate-x-4' : 'translate-x-0'}`} />
            </button>
          </div>
          <div>
            <Label>Etiqueta</Label>
            <Input value={formLabel} onChange={e => setFormLabel(e.target.value)} />
          </div>
          <div className="flex justify-end gap-2 mt-2">
            <Button variant="secondary" size="md" onClick={() => setModalOpen(false)}>Cancelar</Button>
            <Button variant="primary" size="md" onClick={handleSave}>Guardar</Button>
          </div>
        </div>
      </Modal>
    </div>
  )
}
