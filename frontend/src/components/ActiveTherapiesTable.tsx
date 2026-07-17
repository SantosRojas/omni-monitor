import { useEffect, useState, useCallback, useMemo } from 'react'
import { MessageSquare, MessageSquarePlus, Trash2 } from 'lucide-react'
import type { ActiveTherapy, TherapyComment } from '../types'
import { getActiveTherapies } from '../api/patients'
import { createTherapyComment, deleteTherapyComment } from '../api/comments'
import { generateToken } from '../api/auth'
import { getConfig, type AppConfig } from '../api/config'
import { listSignals } from '../api/signals'
import { Button, Modal } from './ui'
import { DataTable } from './data-table/DataTable'
import { useAuth } from '../contexts/AuthContext'
import { useToast } from '../contexts/ToastContext'
import { formatDate } from '../utils/date'
import {
  createColumnHelper,
  useReactTable,
  getCoreRowModel,
  getFilteredRowModel,
  type ColumnFiltersState,
} from '@tanstack/react-table'

const columnHelper = createColumnHelper<ActiveTherapy>()

/** Maps column accessor names → actual signal internal_name in the database */
const SIGNAL_INTERNAL_NAMES: Record<string, string> = {
  arterial_pressure: 'c_press_ap_act',
  venous_pressure: 'c_press_vp_act',
  filter_pressure: 'c_press_fp_act',
  tmp_pressure: 'c_press_tmp_act',
  effluent_pressure: 'c_press_ep_act',
  blood_flow: 'c_pump_bs_bl_flow_act',
  net_rem_flow: 'c_net_rem_flow_act',
  fs_mid_flow: 'c_pump_fs_mid_flow_act',
}

const hideSm = (id: string) =>
  ['time', 'venous_pressure', 'filter_pressure', 'tmp_pressure', 'effluent_pressure', 'net_rem_flow', 'fs_mid_flow'].includes(id)
    ? 'hidden md:table-cell'
    : ''

function calcDuration(startedAt?: string): string {
  if (!startedAt) return '-'
  const st = startedAt.endsWith('Z') ? startedAt : startedAt + 'Z'
  const start = new Date(st).getTime()
  const now = Date.now()
  const diff = now - start
  if (diff < 0) return '-'
  const hours = Math.floor(diff / 3600000)
  const minutes = Math.floor((diff % 3600000) / 60000)
  return `${hours}h ${minutes}m`
}

interface CommentModalProps {
  therapy: ActiveTherapy
  open: boolean
  onClose: () => void
  canWrite: boolean
}

function CommentModal({ therapy, open, onClose, canWrite }: CommentModalProps) {
  const { showToast } = useToast()
  const [comments, setComments] = useState<TherapyComment[]>(therapy.comments)
  const [newComment, setNewComment] = useState('')
  const [sending, setSending] = useState(false)

  useEffect(() => {
    setComments(therapy.comments)
  }, [therapy.comments])

  const handleAdd = async () => {
    if (!newComment.trim()) return
    setSending(true)
    try {
      const created = await createTherapyComment(therapy.therapy_id, newComment.trim())
      setComments(prev => [...prev, created])
      setNewComment('')
    } catch (e) {
      showToast(e instanceof Error ? e.message : 'Error al agregar comentario')
    } finally {
      setSending(false)
    }
  }

  const handleDelete = async (commentId: number) => {
    try {
      await deleteTherapyComment(therapy.therapy_id, commentId, 'Eliminado por usuario')
      setComments(prev => prev.filter(c => c.id !== commentId))
    } catch (e) {
      showToast(e instanceof Error ? e.message : 'Error al eliminar comentario')
    }
  }

  return (
    <Modal open={open} onClose={onClose} title={`Comentarios - ${therapy.patient_id_str}`}>
      <div className="space-y-3 max-h-60 overflow-y-auto mb-3">
        {comments.length === 0 && (
          <p className="text-sm text-(--text-muted) text-center">Sin comentarios</p>
        )}
        {comments.map(c => (
          <div key={c.id} className="flex items-start justify-between gap-2 p-2 rounded-sm bg-(--surface-row-hover)">
            <div className="flex-1 min-w-0">
              <div className="flex items-center gap-2 mb-0.5">
                <span className="text-xs font-semibold text-(--text-primary)">{c.author_name}</span>
                <span className="text-xs text-(--text-muted)">{c.created_at ? formatDate(c.created_at) : ''}</span>
              </div>
              <p className="text-sm text-(--text-secondary) wrap-break-word">{c.comment}</p>
            </div>
            {canWrite && (
              <button
                onClick={() => handleDelete(c.id)}
                className="p-1 text-(--text-muted) hover:text-(--danger) cursor-pointer shrink-0"
                title="Eliminar comentario"
              >
                <Trash2 className="w-3.5 h-3.5" />
              </button>
            )}
          </div>
        ))}
      </div>
      {canWrite && (
        <div className="flex gap-2 items-end">
          <textarea
            value={newComment}
            onChange={e => setNewComment(e.target.value)}
            placeholder="Escribir comentario..."
            rows={4}
            className="flex-1 px-3 py-2 text-sm rounded-sm border border-(--glass-border) bg-(--surface-input) text-(--text-primary) outline-none focus:border-(--accent) transition-colors resize-none"
            disabled={sending}
          />
          <Button variant="primary" size="sm" onClick={handleAdd} disabled={!newComment.trim() || sending}>
            <MessageSquarePlus className="w-4 h-4" />
          </Button>
        </div>
      )}
    </Modal>
  )
}

export function ActiveTherapiesTable() {
  const { showToast } = useToast()
  const { user } = useAuth()
  const [therapies, setTherapies] = useState<ActiveTherapy[]>([])
  const [loading, setLoading] = useState(true)
  const [config, setConfig] = useState<AppConfig>({ polling_interval_ms: 15000 })
  const [commentTarget, setCommentTarget] = useState<ActiveTherapy | null>(null)
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
  const [signalsMap, setSignalsMap] = useState<Record<string, string | undefined>>({})

  const openMachine = useCallback(async (therapy: ActiveTherapy) => {
    if (!therapy.ip_address || !user) return
    try {
      const res = await generateToken(user.id)
      const baseUrl = therapy.port
        ? `http://${therapy.ip_address}:${therapy.port}`
        : `http://${therapy.ip_address}`
      window.open(`${baseUrl}/therapy/${therapy.therapy_id}?token_permanente=${res.code}`, '_blank')
    } catch {
      showToast('Error al generar token de acceso')
    }
  }, [user, showToast])

  const fetchData = useCallback(async () => {
    try {
      const data = await getActiveTherapies()
      setTherapies(data)
    } catch (e) {
      showToast(e instanceof Error ? e.message : 'Error al cargar terapias activas')
    } finally {
      setLoading(false)
    }
  }, [showToast])

  useEffect(() => {
    getConfig()
      .then(c => setConfig(c))
      .catch(() => { })
    listSignals()
      .then(signals => {
        const map: Record<string, string | undefined> = {}
        for (const s of signals) {
          map[s.internal_name] = s.unit
        }
        setSignalsMap(map)
      })
      .catch(() => { })
  }, [])

  useEffect(() => {
    fetchData()
    const id = setInterval(fetchData, config.polling_interval_ms)
    return () => clearInterval(id)
  }, [fetchData, config.polling_interval_ms])

  const columns = useMemo(() => {
    const h = (base: string, accessorName: string) => {
      const internalName = SIGNAL_INTERNAL_NAMES[accessorName]
      const u = internalName ? signalsMap[internalName] : undefined
      return u ? `${base} (${u})` : base
    }

    return [
      columnHelper.display({
        id: 'patient',
        header: 'Paciente',
        cell: ({ row }) => (
          <button
            onClick={() => openMachine(row.original)}
            className="p-1 text-(--text-muted) hover:text-(--accent) cursor-pointer hover:scale-150 transition-transform"
            title="Ver detalle"
          >
            {row.original.patient_id_str}
          </button>
        )
      }),

      columnHelper.accessor('arterial_pressure', {
        header: h('P. Arterial', 'arterial_pressure'),
        cell: i => i.getValue() ?? '-',
      }),
      columnHelper.accessor('venous_pressure', {
        header: h('P. Venosa', 'venous_pressure'),
        cell: i => i.getValue() ?? '-',
      }),
      columnHelper.accessor('filter_pressure', {
        header: h('P. Filtro', 'filter_pressure'),
        cell: i => i.getValue() ?? '-',
      }),
      columnHelper.accessor('tmp_pressure', {
        header: h('TMP', 'tmp_pressure'),
        cell: i => i.getValue() ?? '-',
      }),
      columnHelper.accessor('effluent_pressure', {
        header: h('P. Efluente', 'effluent_pressure'),
        cell: i => i.getValue() ?? '-',
      }),
      columnHelper.accessor('blood_flow', {
        header: h('Flujo Sangre', 'blood_flow'),
        cell: i => i.getValue() ?? '-',
      }),
      columnHelper.accessor('net_rem_flow', {
        header: h('Flujo Rem. Neto', 'net_rem_flow'),
        cell: i => i.getValue() ?? '-',
      }),
      columnHelper.accessor('fs_mid_flow', {
        header: h('Flujo Diálisis', 'fs_mid_flow'),
        cell: i => i.getValue() ?? '-',
      }),
      columnHelper.display({
        id: 'comments',
        header: 'Comentarios',
        cell: ({ row }) => (
          <button
            onClick={() => setCommentTarget(row.original)}
            className="inline-flex items-center gap-1 text-xs px-2 py-1 rounded-sm border border-(--glass-border) bg-(--surface-btn) hover:bg-(--surface-btn-hover) cursor-pointer"
          >
            <MessageSquare className="w-3.5 h-3.5" />
            {row.original.comments.length > 0 ? `${row.original.comments.length}` : '0'}
          </button>
        ),
      }),
      columnHelper.accessor('started_at', {
        header: 'Inicio',
        cell: i => (i.getValue() ? formatDate(i.getValue()!) : '-'),
      }),
      columnHelper.display({
        id: 'time',
        header: 'Tiempo',
        cell: ({ row }) => <span className="font-medium">{calcDuration(row.original.started_at)}</span>,
      })
    ]
  }, [openMachine, signalsMap])

  const table = useReactTable({
    data: therapies,
    columns,
    state: { columnFilters },
    onColumnFiltersChange: setColumnFilters,
    getCoreRowModel: getCoreRowModel(),
    getFilteredRowModel: getFilteredRowModel(),
  })

  if (loading && therapies.length === 0) {
    return (
      <DataTable table={table} loading={loading}>
        <DataTable.Grid />
      </DataTable>
    )
  }

  if (therapies.length === 0) return null

  return (
    <>
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-sm font-semibold uppercase tracking-wider text-(--text-muted)">
          Pacientes con terapia activa
        </h2>
        <span className="text-xs text-(--text-muted)">{therapies.length} activos</span>
      </div>
      <DataTable table={table} loading={loading}>
        <DataTable.Grid
          emptyMessage="Sin pacientes con terapia activa"
          hideSm={hideSm}
        />
      </DataTable>

      {commentTarget && (
        <CommentModal
          therapy={commentTarget}
          open={!!commentTarget}
          onClose={() => setCommentTarget(null)}
          canWrite={user?.role !== 'viewer'}
        />
      )}
    </>
  )
}
