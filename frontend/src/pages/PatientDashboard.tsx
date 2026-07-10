import { useEffect, useMemo, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft } from 'lucide-react'
import type { DashboardSignal } from '../types'
import * as patientsApi from '../api/patients'
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

export function PatientDashboard() {
  const { showToast } = useToast()
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [signals, setSignals] = useState<DashboardSignal[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (!id) return
    patientsApi.getPatientDashboard(Number(id))
      .then(res => setSignals(res.signals))
      .catch(e => showToast(e instanceof Error ? e.message : 'Error al cargar dashboard'))
      .finally(() => setLoading(false))
  }, [id])

  const pressureSignals = useMemo(
    () => signals.filter(s => PRESSURE_SIGNALS.has(s.internal_name)),
    [signals]
  )

  const flowSignals = useMemo(
    () => signals.filter(s => FLOW_SIGNALS.has(s.internal_name)),
    [signals]
  )

  return (
    <div>
      <button onClick={() => navigate(`/patients/${id}`)} className="flex items-center gap-1.5 text-sm text-(--text-secondary) hover:text-(--text-primary) mb-4 cursor-pointer">
        <ArrowLeft className="w-4 h-4" /> Volver al paciente
      </button>

      <h2 className="text-xl font-bold mb-5 text-(--text-primary)">Dashboard de Señales</h2>

      {loading ? <Spinner message="Cargando dashboard..." /> : (
        <>
          {pressureSignals.length > 0 && <Chart title="Presiones" signals={pressureSignals} />}
          {flowSignals.length > 0 && <Chart title="Flujos" signals={flowSignals} />}
          {pressureSignals.length === 0 && flowSignals.length === 0 && (
            <div className="text-center py-10 text-(--text-muted) text-sm">Sin señales disponibles</div>
          )}
        </>
      )}
    </div>
  )
}
