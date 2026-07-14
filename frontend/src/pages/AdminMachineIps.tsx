import { useEffect, useMemo, useState } from 'react'
import {
  useReactTable,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  createColumnHelper,
} from '@tanstack/react-table'
import type { ColumnFiltersState, SortingState } from '@tanstack/react-table'
import { Plus, Pencil, Trash2, ExternalLink } from 'lucide-react'
import type { MachineIpWithSerial, Machine } from '../types'
import * as machinesApi from '../api/machines'
import { generateToken } from '../api/auth'
import { Modal, Badge, Combobox, Button, Input, Label } from '../components/ui'
import { DataTable } from '../components/data-table'
import { useToast } from '../contexts/ToastContext'
import { useAuth } from '../contexts/AuthContext'

const helper = createColumnHelper<MachineIpWithSerial>()

const hideSm = (id: string) =>
  ['port', 'label'].includes(id) ? 'hidden md:table-cell' : ''

export function AdminMachineIps() {
  const { showToast } = useToast()
  const { user } = useAuth()
  const [data, setData] = useState<MachineIpWithSerial[]>([])
  const [machines, setMachines] = useState<Machine[]>([])
  const [loading, setLoading] = useState(true)
  const [modalOpen, setModalOpen] = useState(false)
  const [editing, setEditing] = useState<MachineIpWithSerial | null>(null)

  const [formMachineId, setFormMachineId] = useState(0)
  const [formSerialNumber, setFormSerialNumber] = useState('')
  const [formIp, setFormIp] = useState('')
  const [formPort, setFormPort] = useState<number | undefined>(undefined)
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
    setFormMachineId(0)
    setFormSerialNumber('')
    setFormIp('')
    setFormPort(undefined)
    setFormLabel('')
    setFormIsActive(true)
    setModalOpen(true)
  }

  const openEdit = (item: MachineIpWithSerial) => {
    setEditing(item)
    setFormMachineId(item.machine_id)
    setFormIp(item.ip_address)
    setFormPort(item.port ?? undefined)
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
          serial_number: formMachineId === 0 ? formSerialNumber : undefined,
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

  const openMachineUrl = async (ip: string, port?: number) => {
    if (!ip || !user) return
    try {
      const res = await generateToken(user.id)
      const baseUrl = port ? `http://${ip}:${port}` : `http://${ip}`
      window.open(`${baseUrl}/?token_permanente=${res.code}`, '_blank')
    } catch {
      showToast('Error al generar token de acceso')
    }
  }

  const columns = useMemo(() => [
    helper.accessor('serial_number', { header: 'Serial' }),
    helper.accessor('ip_address', { header: 'Dirección IP' }),
    helper.accessor('port', { header: 'Puerto', cell: i => i.getValue() ?? '—' }),
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
          <Button variant="icon" size="sm" onClick={() => openMachineUrl(row.original.ip_address, row.original.port)}>
            <ExternalLink className="w-4 h-4" />
          </Button>
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
    defaultColumn: { filterFn: 'includesString' },
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

      <DataTable table={table} loading={loading}>
        <DataTable.Search />
        <DataTable.Grid
          emptyMessage="No hay IPs registradas"
          hideSm={hideSm}
        />
      </DataTable>

      <Modal open={modalOpen} onClose={() => setModalOpen(false)} title={editing ? 'Editar IP' : 'Nueva IP'}>
        <div className="flex flex-col gap-4">
          {!editing && (
            <div>
              <Label>Máquina</Label>
              <Combobox
                options={machines.map(m => ({ value: m.id, label: m.serial_number }))}
                value={formMachineId}
                onChange={(id, label) => {
                  setFormMachineId(id)
                  if (id === 0) setFormSerialNumber(label)
                  else setFormSerialNumber('')
                }}
                placeholder="Buscar o escribir serial…"
              />
            </div>
          )}
          <div>
            <Label>Dirección IP</Label>
            <Input value={formIp} onChange={e => setFormIp(e.target.value)} />
          </div>
          <div>
            <Label>Puerto</Label>
            <Input type="number" value={formPort ?? ''} onChange={e => setFormPort(e.target.value === '' ? undefined : Number(e.target.value))} />
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
