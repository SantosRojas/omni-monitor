import { useEffect, useMemo, useState } from 'react'
import { useParams, useNavigate, Link } from 'react-router-dom'
import {
  useReactTable,
  getCoreRowModel,
  getFilteredRowModel,
  getSortedRowModel,
  createColumnHelper,
} from '@tanstack/react-table'
import type { ColumnFiltersState, SortingState } from '@tanstack/react-table'
import { ArrowLeft, FileDown, LineChart, Clock } from 'lucide-react'
import type { Patient, TherapyWithMachine } from '../types'
import * as patientsApi from '../api/patients'
import { triggerPatientExport } from '../api/export'
import { Spinner, Badge, ColumnFilter, Button, SearchInput } from '../components/ui'
import { formatDate, formatDateShort } from '../utils/date'

const therapyHelper = createColumnHelper<TherapyWithMachine>()

const hideSm = (id: string) =>
  ['ended_at', 'software_version'].includes(id) ? 'hidden md:table-cell' : ''

export function PatientDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [patient, setPatient] = useState<Patient | null>(null)
  const [therapies, setTherapies] = useState<TherapyWithMachine[]>([])
  const [loading, setLoading] = useState(true)
  const [sorting, setSorting] = useState<SortingState>([])
  const [columnFilters, setColumnFilters] = useState<ColumnFiltersState>([])
  const [globalFilter, setGlobalFilter] = useState('')

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
    getFilteredRowModel: getFilteredRowModel(),
    getSortedRowModel: getSortedRowModel(),
    onSortingChange: setSorting,
    onColumnFiltersChange: setColumnFilters,
    onGlobalFilterChange: setGlobalFilter,
    globalFilterFn: 'includesString',
    state: { sorting, columnFilters, globalFilter },
  })

  if (loading) return <Spinner message="Cargando paciente..." />
  if (!patient) return <div className="text-center py-10 text-(--text-muted)">Paciente no encontrado</div>

  return (
    <div>
      <Button variant="ghost" size="sm" onClick={() => navigate('/patients')} icon={<ArrowLeft className="w-4 h-4" />}>
        Volver a pacientes
      </Button>

      <div className="glass p-4 md:p-5 mb-5">
        <div className="flex flex-col sm:flex-row items-start justify-between gap-3 mb-3">
          <div>
            <h2 className='text-(--text-primary)'><span className="text-lg md:text-xl font-bold">Paciente: </span> {patient.patient_id_str}</h2>
            {/* <p className="text-sm text-(--text-muted) mt-1">ID: {patient.id}</p> */}
          </div>
          <div className="flex flex-wrap gap-2">
            <Link to={`/patients/${patient.id}/dashboard`} className="no-underline">
              <Button variant="secondary" size="sm" icon={<LineChart className="w-4 h-4" />}>Dashboard</Button>
            </Link>
            <Link to={`/patients/${patient.id}/history`} className="no-underline">
              <Button variant="secondary" size="sm" icon={<Clock className="w-4 h-4" />}>Historial</Button>
            </Link>
            <Button variant="secondary" size="sm" icon={<FileDown className="w-4 h-4" />} onClick={() => triggerPatientExport(patient.id).catch(console.error)}>
              Exportar
            </Button>
          </div>
        </div>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-3 md:gap-4 text-xs md:text-sm">
          <div><span className="text-(--text-muted)">Creado:</span><br />{formatDateShort(patient.created_at)}</div>
          <div><span className="text-(--text-muted)">Id:</span><br />{patient.id}</div>
          <div><span className="text-(--text-muted)">Terapias Activas:</span><br /><Badge variant={patient.active_therapy_count && patient.active_therapy_count > 0 ? 'active' : 'inactive'}>{patient.active_therapy_count ?? 0}</Badge></div>
          <div><span className="text-(--text-muted)">Terapias completas:</span><br /><Badge variant={'active'}>{patient.completed_therapy_count ?? 0}</Badge></div>
        </div>
      </div>

      <div className="flex flex-col sm:flex-row items-start sm:items-center gap-3 mb-3">
        <h3 className="text-base md:text-lg font-semibold text-(--text-primary)">Terapias</h3>
        <div className="w-full sm:w-64 sm:ml-auto">
          <SearchInput value={globalFilter ?? ''} onChange={e => setGlobalFilter(e.target.value)} placeholder="Buscar en toda la tabla..." />
        </div>
      </div>
      {therapies.length === 0 ? (
        <div className="text-center py-10 text-(--text-muted) text-sm">Sin terapias registradas</div>
      ) : (
        <div className="glass overflow-x-auto">
          <table className="w-full border-collapse">
            <thead>
              {therapyTable.getHeaderGroups().map(hg => (
                <tr key={hg.id}>
                  {hg.headers.map(h => (
                    <th key={h.id} onClick={h.column.getToggleSortingHandler()} className={`text-left px-4 py-3 text-xs font-semibold uppercase tracking-wider text-(--text-muted) border-b border-(--border-subtle) cursor-pointer select-none ${hideSm(h.id)}`}>
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
              {therapyTable.getRowModel().rows.map(row => (
                <tr
                  key={row.id}
                  onClick={() => {
                    const t = row.original
                    if (t.status === 'active') {
                      if (t.ip_address) {
                        window.open(`http://${t.ip_address}:${t.port ?? 9001}/therapy/${t.id}`, '_blank')
                      } else {
                        alert('No se encuentra la IP de la máquina registrada')
                      }
                    } else {
                      navigate(`/therapies/${t.id}`)
                    }
                  }}
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
