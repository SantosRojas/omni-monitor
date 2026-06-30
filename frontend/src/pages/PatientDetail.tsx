import { useEffect, useMemo, useState } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import {
  useReactTable,
  getCoreRowModel,
  createColumnHelper,
} from '@tanstack/react-table'
import { ArrowLeft, FileDown, LineChart, Clock } from 'lucide-react'
import type { Patient, TherapyWithMachine } from '../types'
import * as patientsApi from '../api/patients'
import { triggerPatientExport } from '../api/export'
import { Spinner } from '../components/ui/Spinner'
import { Badge } from '../components/ui/Badge'
import { formatDate, formatDateShort } from '../utils/date'

const therapyHelper = createColumnHelper<TherapyWithMachine>()

const hideSm = (id: string) =>
  ['id', 'ended_at', 'software_version'].includes(id) ? 'hidden md:table-cell' : ''

export function PatientDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [patient, setPatient] = useState<Patient | null>(null)
  const [therapies, setTherapies] = useState<TherapyWithMachine[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (!id) return
    const pid = Number(id)
    Promise.all([
      patientsApi.getPatient(pid),
      patientsApi.getTherapies(pid),
    ]).then(([p, t]) => {
      setPatient(p)
      setTherapies(t)
    }).catch(console.error)
      .finally(() => setLoading(false))
  }, [id])

  const therapyColumns = useMemo(() => [
    therapyHelper.accessor('id', { header: 'ID' }),
    therapyHelper.accessor('started_at', {
      header: 'Inicio',
      cell: i => formatDate(i.getValue()),
    }),
    therapyHelper.accessor('ended_at', {
      header: 'Fin',
      cell: i => formatDate(i.getValue()),
    }),
    therapyHelper.accessor('status', {
      header: 'Estado',
      cell: i => {
        const v = i.getValue()
        if (v === 'active') return <Badge variant="active">Activo</Badge>
        if (v === 'completed') return <Badge variant="completed">Completada</Badge>
        return <Badge variant="inactive">{v || '-'}</Badge>
      },
    }),
    therapyHelper.accessor('serial_number', { header: 'Máquina' }),
    therapyHelper.accessor('software_version', { header: 'Versión' }),
  ], [])

  const therapyTable = useReactTable({
    data: therapies,
    columns: therapyColumns,
    getCoreRowModel: getCoreRowModel(),
  })

  if (loading) return <Spinner message="Cargando paciente..." />
  if (!patient) return <div className="text-center py-10 text-(--text-muted)">Paciente no encontrado</div>

  return (
    <div>
      <button onClick={() => navigate('/patients')} className="flex items-center gap-1.5 text-sm text-(--text-secondary) hover:text-(--text-primary) mb-4 cursor-pointer">
        <ArrowLeft className="w-4 h-4" /> Volver a pacientes
      </button>

      <div className="glass p-4 md:p-5 mb-5">
        <div className="flex flex-col sm:flex-row items-start justify-between gap-3 mb-3">
          <div>
            <h2 className='text-(--text-primary)'><span className="text-lg md:text-xl font-bold">Paciente: </span> {patient.patient_id_str}</h2>
            {/* <p className="text-sm text-(--text-muted) mt-1">ID: {patient.id}</p> */}
          </div>
          <div className="flex flex-wrap gap-2">
            <Link to={`/patients/${patient.id}/dashboard`} className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) no-underline">
              <LineChart className="w-4 h-4" /> Dashboard
            </Link>
            <Link to={`/patients/${patient.id}/history`} className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) no-underline">
              <Clock className="w-4 h-4" /> Historial
            </Link>
            <button onClick={() => triggerPatientExport(patient.id).catch(console.error)} className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) cursor-pointer">
              <FileDown className="w-4 h-4" /> Exportar
            </button>
          </div>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 md:gap-4 text-xs md:text-sm">
          <div><span className="text-(--text-muted)">Creado:</span><br />{formatDateShort(patient.created_at)}</div>
          <div><span className="text-(--text-muted)">Id:</span><br />{patient.id}</div>
          <div><span className="text-(--text-muted)">Terapias Activas:</span><br /><Badge variant={patient.active_therapy_count && patient.active_therapy_count > 0 ? 'active' : 'inactive'}>{patient.active_therapy_count ?? 0}</Badge></div>
          <div><span className="text-(--text-muted)">Terapias completas:</span><br /><Badge variant={'active'}>{patient.completed_therapy_count ?? 0}</Badge></div>
        </div>
      </div>

      <h3 className="text-base md:text-lg font-semibold mb-3 text-(--text-primary)">Terapias</h3>
      {therapies.length === 0 ? (
        <div className="text-center py-10 text-(--text-muted) text-sm">Sin terapias registradas</div>
      ) : (
        <div className="glass overflow-x-auto">
          <table className="w-full border-collapse">
            <thead>
              {therapyTable.getHeaderGroups().map(hg => (
                <tr key={hg.id}>
                  {hg.headers.map(h => (
                    <th key={h.id} className={`text-left px-4 py-3 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle) ${hideSm(h.id)}`}>
                      {h.column.columnDef.header as string}
                    </th>
                  ))}
                </tr>
              ))}
            </thead>
            <tbody>
              {therapyTable.getRowModel().rows.map(row => (
                <tr
                  key={row.id}
                  onClick={() => navigate(`/therapies/${row.original.id}`)}
                  className="cursor-pointer hover:bg-(--surface-row-hover) transition-colors"
                >
                  {row.getVisibleCells().map(cell => (
                    <td key={cell.id} className={`px-4 py-3 text-sm text-(--text-secondary) border-b border-(--border-subtle) ${hideSm(cell.column.id)}`}>
                      {cell.column.columnDef.cell ? (cell.column.columnDef.cell as any)(cell.getContext()) : cell.getValue() as string}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
