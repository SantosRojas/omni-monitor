import { useEffect, useMemo, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, FileDown } from 'lucide-react'
import type { DashboardSignal, TherapyWithMachine } from '../types'
import * as patientsApi from '../api/patients'
import { triggerTherapyExport } from '../api/export'
import { Spinner } from '../components/ui/Spinner'
import { Chart } from '../components/Chart'
import { useToast } from '../contexts/ToastContext'

const PRESSURE_SIGNALS = new Set([
  'c_press_ap_act',
  'c_press_vp_act',
  'c_press_fp_act',
  'c_press_tmp_act',
  'c_press_ep_act',
])

const FLOW_SIGNALS = new Set([
  'c_pump_bs_bl_flow_act',
  'c_net_rem_flow_act',
  'c_pump_fs_mid_flow_act',
])

const WEIGHT_SIGNALS = new Set([
  'g_patient_data_weight_set',
])

const VOLUME_SIGNALS = new Set([
  'c_acc_net_rem_vol_act',
])

export function TherapyDetail() {
  const { showToast } = useToast()
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [signals, setSignals] = useState<DashboardSignal[]>([])
  const [therapy, setTherapy] = useState<TherapyWithMachine | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (!id) return
    const tid = Number(id)
    const prevTitle = document.title
    Promise.all([
      patientsApi.getTherapyDashboard(tid),
      patientsApi.getTherapy(tid),
    ]).then(([s, t]) => {
      setSignals(s.signals)
      setTherapy(t)
      const title = [t.patient_id_str, t.serial_number].filter(Boolean).join(' - ')
      if (title) document.title = title
    }).catch(e => showToast(e instanceof Error ? e.message : 'Error al cargar terapia'))
      .finally(() => setLoading(false))
    return () => { document.title = prevTitle }
  }, [id])

  const pressureSignals = useMemo(
    () => signals.filter(s => PRESSURE_SIGNALS.has(s.internal_name)),
    [signals]
  )

  const flowSignals = useMemo(
    () => signals.filter(s => FLOW_SIGNALS.has(s.internal_name)),
    [signals]
  )

  const weightSignals = useMemo(
    () => signals.filter(s => WEIGHT_SIGNALS.has(s.internal_name)),
    [signals]
  )

  const volumeSignals = useMemo(
    () => signals.filter(s => VOLUME_SIGNALS.has(s.internal_name)),
    [signals]
  )

  return (
    <div>
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 mb-5">
        <div className='flex gap-1'>
          <button onClick={() => navigate(-1)} className="px-3 py-1.5 flex items-center gap-1.5 text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) cursor-pointer">
            <ArrowLeft className="w-4 h-4" />
          </button>
          <h2 className="text-lg md:text-xl font-bold text-(--text-primary)">
            {therapy ? `${therapy.patient_id_str ?? ''}${therapy.patient_id_str && therapy.serial_number ? ' - ' : ''}${therapy.serial_number ?? ''}` : `Terapia #${id}`}
          </h2>
        </div>
        <button onClick={() => triggerTherapyExport(Number(id)).catch(e => showToast(e instanceof Error ? e.message : 'Error al exportar'))} className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) cursor-pointer">
          <FileDown className="w-4 h-4" /> Exportar
        </button>
      </div>

      {loading ? <Spinner message="Cargando terapia..." /> : (
        <>
          {pressureSignals.length > 0 && <Chart title="Presiones" signals={pressureSignals} />}
          {flowSignals.length > 0 && <Chart title="Flujos" signals={flowSignals} />}
          {weightSignals.length > 0 && <Chart title="Peso" signals={weightSignals} />}
          {volumeSignals.length > 0 && <Chart title="Volumen Neto" signals={volumeSignals} />}
          {pressureSignals.length === 0 && flowSignals.length === 0 && weightSignals.length === 0 && volumeSignals.length === 0 && (
            <div className="text-center py-10 text-(--text-muted) text-sm">Sin datos de señales para esta terapia</div>
          )}
        </>
      )}
    </div>
  )
}
