import { useEffect, useState, useCallback } from 'react'
import { MessageSquare, MessageSquarePlus, Trash2, ExternalLink } from 'lucide-react'
import type { ActiveTherapy, TherapyComment } from '../types'
import { getActiveTherapies } from '../api/patients'
import { createTherapyComment, deleteTherapyComment } from '../api/comments'
import { getConfig, type AppConfig } from '../api/config'
import { Spinner, Button, Modal } from './ui'
import { useToast } from '../contexts/ToastContext'
import { formatDate } from '../utils/date'

function calcDuration(startedAt?: string): string {
  if (!startedAt) return '-'
  const start = new Date(startedAt + 'Z').getTime()
  const now = Date.now()
  const diff = now - start
  if (diff < 0) return '-'
  const hours = Math.floor(diff / 3600000)
  const minutes = Math.floor((diff % 3600000) / 60000)
  return `${hours}h ${minutes}m`
}

function openMachine(therapy: ActiveTherapy) {
  if (!therapy.ip_address) return
  const baseUrl = therapy.port
    ? `http://${therapy.ip_address}:${therapy.port}`
    : `http://${therapy.ip_address}`
  window.open(`${baseUrl}/therapy/${therapy.therapy_id}`, '_blank')
}

interface CommentModalProps {
  therapy: ActiveTherapy
  open: boolean
  onClose: () => void
}

function CommentModal({ therapy, open, onClose }: CommentModalProps) {
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
            <button
              onClick={() => handleDelete(c.id)}
              className="p-1 text-(--text-muted) hover:text-(--danger) cursor-pointer shrink-0"
              title="Eliminar comentario"
            >
              <Trash2 className="w-3.5 h-3.5" />
            </button>
          </div>
        ))}
      </div>
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
    </Modal>
  )
}

export function ActiveTherapiesTable() {
  const { showToast } = useToast()
  const [therapies, setTherapies] = useState<ActiveTherapy[]>([])
  const [loading, setLoading] = useState(true)
  const [config, setConfig] = useState<AppConfig>({ polling_interval_ms: 15000 })
  const [commentTarget, setCommentTarget] = useState<ActiveTherapy | null>(null)

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
  }, [])

  useEffect(() => {
    fetchData()
    const id = setInterval(fetchData, config.polling_interval_ms)
    return () => clearInterval(id)
  }, [fetchData, config.polling_interval_ms])

  if (loading) return <Spinner message="Cargando terapias activas..." />

  if (therapies.length === 0) return null

  return (
    <>
      <div className="glass overflow-x-auto mb-4">
        <div className="flex items-center justify-between gap-3 px-4 py-3">
          <h2 className="text-sm font-semibold uppercase tracking-wider text-(--text-muted)">
            Pacientes con terapia activa
          </h2>
          <span className="text-xs text-(--text-muted)">{therapies.length} activos</span>
        </div>
        <table className="w-full border-collapse">
          <thead>
            <tr>
              <th className="text-left px-3 py-2 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle)">Inicio</th>
              <th className="text-left px-3 py-2 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle)">Tiempo</th>
              <th className="text-left px-3 py-2 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle)">Paciente</th>
              <th className="text-left px-3 py-2 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle)">P. Arterial</th>
              <th className="text-left px-3 py-2 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle)">P. Venosa</th>
              <th className="text-left px-3 py-2 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle)">Flujo Sangre</th>
              <th className="text-left px-3 py-2 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle)">Peso Inicial</th>
              <th className="text-left px-3 py-2 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle)">Peso Actual</th>
              <th className="text-left px-3 py-2 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle)">Comentarios</th>
              <th className="text-left px-3 py-2 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle)">Máquina</th>
            </tr>
          </thead>
          <tbody>
            {therapies.map(t => (
              <tr key={t.therapy_id} className="hover:bg-(--surface-row-hover) transition-colors">
                <td className="px-3 py-2 text-sm text-(--text-secondary) border-b border-(--border-subtle) whitespace-nowrap">
                  {t.started_at ? formatDate(t.started_at) : '-'}
                </td>
                <td className="px-3 py-2 text-sm text-(--text-secondary) border-b border-(--border-subtle) whitespace-nowrap font-medium">
                  {calcDuration(t.started_at)}
                </td>
                <td className="px-3 py-2 text-sm text-(--text-secondary) border-b border-(--border-subtle) whitespace-nowrap">
                  {t.patient_id_str}
                </td>
                <td className="px-3 py-2 text-sm text-(--text-secondary) border-b border-(--border-subtle) whitespace-nowrap">
                  {t.arterial_pressure ?? '-'}
                </td>
                <td className="px-3 py-2 text-sm text-(--text-secondary) border-b border-(--border-subtle) whitespace-nowrap">
                  {t.venous_pressure ?? '-'}
                </td>
                <td className="px-3 py-2 text-sm text-(--text-secondary) border-b border-(--border-subtle) whitespace-nowrap">
                  {t.blood_flow ?? '-'}
                </td>
                <td className="px-3 py-2 text-sm text-(--text-secondary) border-b border-(--border-subtle) whitespace-nowrap">
                  {t.weight_initial ?? '-'}
                </td>
                <td className="px-3 py-2 text-sm text-(--text-secondary) border-b border-(--border-subtle) whitespace-nowrap">
                  {t.weight_final ?? '-'}
                </td>
                <td className="px-3 py-2 text-sm text-(--text-secondary) border-b border-(--border-subtle)">
                  <button
                    onClick={() => setCommentTarget(t)}
                    className="inline-flex items-center gap-1 text-xs px-2 py-1 rounded-sm border border-(--glass-border) bg-(--surface-btn) hover:bg-(--surface-btn-hover) cursor-pointer"
                  >
                    <MessageSquare className="w-3.5 h-3.5" />
                    {t.comments.length > 0 ? `${t.comments.length}` : '0'}
                  </button>
                </td>
                <td className="px-3 py-2 text-sm border-b border-(--border-subtle)">
                  {t.ip_address && (
                    <button
                      onClick={() => openMachine(t)}
                      className="p-1 text-(--text-muted) hover:text-(--accent) cursor-pointer"
                      title="Abrir máquina"
                    >
                      <ExternalLink className="w-4 h-4" />
                    </button>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>

      {commentTarget && (
        <CommentModal
          therapy={commentTarget}
          open={!!commentTarget}
          onClose={() => setCommentTarget(null)}
        />
      )}
    </>
  )
}
