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
import { Modal, Badge, Select, Button, Input, Label } from '../components/ui'
import { DataTable } from '../components/data-table'
import { useToast } from '../contexts/ToastContext'

const helper = createColumnHelper<UserResponse>()

const hideSm = (id: string) =>
  ['full_name', 'email'].includes(id) ? 'hidden md:table-cell' : ''

const roleBadge: Record<string, 'admin' | 'operator' | 'viewer'> = {
  admin: 'admin',
  operator: 'operator',
  viewer: 'viewer',
}

export function AdminUsers() {
  const { showToast } = useToast()
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

  const [tableKey, setTableKey] = useState(0)

  const fetchData = () => {
    setLoading(true)
    usersApi.listUsers()
      .then(setData)
      .catch(e => showToast(e instanceof Error ? e.message : 'Error al cargar usuarios'))
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
      showToast(e instanceof Error ? e.message : 'Error al guardar')
    }
  }

  const handleToggleActive = async (user: UserResponse) => {
    try {
      await usersApi.updateUser(user.id, { active: !user.active })
      fetchData()
    } catch (e) {
      showToast(e instanceof Error ? e.message : 'Error al cambiar estado')
    }
  }

  const handleDelete = async (id: number) => {
    if (!confirm('¿Eliminar este usuario?')) return
    try {
      await usersApi.deleteUser(id)
      fetchData()
    } catch (e) {
      showToast(e instanceof Error ? e.message : 'Error al eliminar')
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

  const handleCloseModal = () => {
    setModalOpen(false)
    setTableKey(k => k + 1)
  }

  return (
    <div key={tableKey}>
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 mb-5">
        <h2 className="text-lg md:text-xl font-bold text-(--text-primary)">Usuarios</h2>
        <Button variant="primary" size="sm" icon={<Plus className="w-4 h-4" />} onClick={openCreate}>
          Nuevo Usuario
        </Button>
      </div>

      <DataTable table={table} loading={loading}>
        <DataTable.Search />
        <DataTable.Grid
          emptyMessage="No hay usuarios"
          hideSm={hideSm}
        />
      </DataTable>

      <Modal open={modalOpen} onClose={handleCloseModal} title={editing ? 'Editar Usuario' : 'Nuevo Usuario'}>
        <form onSubmit={e => e.preventDefault()} className="flex flex-col gap-4">
          <div>
            <Label>Usuario</Label>
            <Input autoComplete="username" value={formUsername} onChange={e => setFormUsername(e.target.value)} disabled={!!editing} />
          </div>
          <div>
            <Label>Contraseña {editing ? '(dejar vacío para no cambiar)' : ''}</Label>
            <Input type="password" autoComplete="new-password" value={formPassword} onChange={e => setFormPassword(e.target.value)} />
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
            <Button variant="secondary" size="md" onClick={handleCloseModal}>Cancelar</Button>
            <Button variant="primary" size="md" onClick={handleSave}>Guardar</Button>
          </div>
        </form>
      </Modal>
    </div>
  )
}
