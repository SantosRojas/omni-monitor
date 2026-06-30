import { useEffect, useMemo, useState } from 'react'
import {
  useReactTable,
  getCoreRowModel,
  createColumnHelper,
} from '@tanstack/react-table'
import { Plus, Pencil, Trash2 } from 'lucide-react'
import type { UserResponse } from '../types'
import * as usersApi from '../api/users'
import { Spinner } from '../components/ui/Spinner'
import { Modal } from '../components/ui/Modal'
import { Badge } from '../components/ui/Badge'

const helper = createColumnHelper<UserResponse>()

const hideSm = (id: string) =>
  ['id', 'full_name', 'email'].includes(id) ? 'hidden md:table-cell' : ''

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
    helper.accessor('id', { header: 'ID' }),
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
        <h2 className="text-lg md:text-xl font-bold text-(--text-primary)">Usuarios</h2>
        <button onClick={openCreate} className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-sm bg-[var(--accent)] text-white hover:opacity-90 cursor-pointer">
          <Plus className="w-4 h-4" /> Nuevo Usuario
        </button>
      </div>

      {loading ? <Spinner message="Cargando usuarios..." /> : (
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
          {data.length === 0 && <div className="text-center py-10 text-(--text-muted) text-sm">No hay usuarios</div>}
        </div>
      )}

      <Modal open={modalOpen} onClose={() => setModalOpen(false)} title={editing ? 'Editar Usuario' : 'Nuevo Usuario'}>
        <div className="flex flex-col gap-4">
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Usuario</label>
            <input value={formUsername} onChange={e => setFormUsername(e.target.value)} disabled={!!editing}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Contraseña {editing ? '(dejar vacío para no cambiar)' : ''}</label>
            <input type="password" value={formPassword} onChange={e => setFormPassword(e.target.value)}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Nombre Completo</label>
            <input value={formFullName} onChange={e => setFormFullName(e.target.value)}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Email</label>
            <input type="email" value={formEmail} onChange={e => setFormEmail(e.target.value)}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]" />
          </div>
          <div>
            <label className="block mb-1 text-xs font-medium text-(--text-secondary)">Rol</label>
            <select value={formRole} onChange={e => setFormRole(e.target.value)}
              className="w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none">
              <option value="admin">Admin</option>
              <option value="operator">Operator</option>
              <option value="viewer">Viewer</option>
            </select>
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
