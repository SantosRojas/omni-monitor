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
import type { UserResponse } from '../types'
import * as usersApi from '../api/users'
import { Spinner, Modal, Badge, Select, ColumnFilter, Button, Input, Label, SearchInput } from '../components/ui'

const helper = createColumnHelper<UserResponse>()

const hideSm = (id: string) =>
  ['full_name', 'email'].includes(id) ? 'hidden md:table-cell' : ''

const roleBadge: Record<string, 'admin' | 'operator' | 'viewer'> = {
  admin: 'admin',
  operator: 'operator',
  viewer: 'viewer',
}

export function AdminUsers() {
  const [data, setData] = useState<UserResponse[]>([])
  const [loading, setLoading] = useState(true)
  const [modalOpen, setModalOpen] = useState(false)
  const [editing, setEditing] = useState<UserResponse | null>(null)
  const [sorting, setSorting] = useState<SortingState>([])
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
  const [globalFilter, setGlobalFilter] = useState('')

  const [formUsername, setFormUsername] = useState('')
  const [formPassword, setFormPassword] = useState('')
  const [formFullName, setFormFullName] = useState('')
  const [formEmail, setFormEmail] = useState('')
  const [formRole, setFormRole] = useState('operator')

  const fetchData = () => {
    setLoading(true)
    usersApi.listUsers()
      .then(setData)
      .catch(console.error)
      .finally(() => setLoading(false))
  }

  useEffect(fetchData, [])

  const openCreate = () => {
    setEditing(null)
    setFormUsername('')
    setFormPassword('')
    setFormFullName('')
    setFormEmail('')
    setFormRole('operator')
    setModalOpen(true)
  }

  const openEdit = (user: UserResponse) => {
    setEditing(user)
    setFormUsername(user.username)
    setFormPassword('')
    setFormFullName(user.full_name)
    setFormEmail(user.email)
    setFormRole(user.role)
    setModalOpen(true)
  }

  const handleSave = async () => {
    try {
      if (editing) {
        await usersApi.updateUser(editing.id, {
          full_name: formFullName,
          email: formEmail,
          role: formRole,
          ...(formPassword ? { password: formPassword } : {}),
        })
      } else {
        await usersApi.createUser({
          username: formUsername,
          password: formPassword,
          full_name: formFullName,
          email: formEmail,
          role: formRole,
        })
      }
      setModalOpen(false)
      fetchData()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Error al guardar')
    }
  }

  const handleToggleActive = async (user: UserResponse) => {
    try {
      await usersApi.updateUser(user.id, { active: !user.active })
      fetchData()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Error al cambiar estado')
    }
  }

  const handleDelete = async (id: number) => {
    if (!confirm('¿Eliminar este usuario?')) return
    try {
      await usersApi.deleteUser(id)
      fetchData()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Error al eliminar')
    }
  }

  const columns = useMemo(() => [
    helper.accessor('username', { header: 'Usuario' }),
    helper.accessor('full_name', { header: 'Nombre Completo' }),
    helper.accessor('email', { header: 'Email' }),
    helper.accessor('role', {
      header: 'Rol',
      cell: i => <Badge variant={roleBadge[i.getValue()] || 'default'}>{i.getValue()}</Badge>,
    }),
    helper.accessor('active', {
      header: 'Estado',
      cell: i => (
        <button onClick={() => handleToggleActive(i.row.original)} className="cursor-pointer">
          <Badge variant={i.getValue() ? 'active' : 'inactive'}>{i.getValue() ? 'Activo' : 'Inactivo'}</Badge>
        </button>
      ),
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
        <h2 className="text-lg md:text-xl font-bold text-(--text-primary)">Usuarios</h2>
        <Button variant="primary" size="sm" icon={<Plus className="w-4 h-4" />} onClick={openCreate}>
          Nuevo Usuario
        </Button>
      </div>

      <div className="mb-4">
        <SearchInput value={globalFilter ?? ''} onChange={e => setGlobalFilter(e.target.value)} placeholder="Buscar en toda la tabla..." />
      </div>

      {loading ? <Spinner message="Cargando usuarios..." /> : (
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
          {data.length === 0 && <div className="text-center py-10 text-(--text-muted) text-sm">No hay usuarios</div>}
        </div>
      )}

      <Modal open={modalOpen} onClose={() => setModalOpen(false)} title={editing ? 'Editar Usuario' : 'Nuevo Usuario'}>
        <div className="flex flex-col gap-4">
          <div>
            <Label>Usuario</Label>
            <Input value={formUsername} onChange={e => setFormUsername(e.target.value)} disabled={!!editing} />
          </div>
          <div>
            <Label>Contraseña {editing ? '(dejar vacío para no cambiar)' : ''}</Label>
            <Input type="password" value={formPassword} onChange={e => setFormPassword(e.target.value)} />
          </div>
          <div>
            <Label>Nombre Completo</Label>
            <Input value={formFullName} onChange={e => setFormFullName(e.target.value)} />
          </div>
          <div>
            <Label>Email</Label>
            <Input type="email" value={formEmail} onChange={e => setFormEmail(e.target.value)} />
          </div>
          <div>
            <Label>Rol</Label>
            <Select
              options={[
                { value: 'admin', label: 'Admin' },
                { value: 'operator', label: 'Operator' },
                { value: 'viewer', label: 'Viewer' }
              ]}
              value={formRole}
              onChange={setFormRole}
              placeholder="Seleccionar rol…"
            />
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
