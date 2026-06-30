import { useEffect, useMemo, useState } from 'react'
import { useParams, useNavigate } from 'react-router-dom'
import { ArrowLeft, FileDown } from 'lucide-react'
import type { DashboardSignal } from '../types'
import * as patientsApi from '../api/patients'
import { triggerTherapyExport } from '../api/export'
import { Spinner } from '../components/ui/Spinner'
import { Chart } from '../components/Chart'

const SIGNALS_TO_SHOW = new Set([
  'c_pump_bs_bl_flow_act',
  'c_net_rem_flow_act',
  'c_pump_fs_mid_flow_act',
  'd_renal_dose_act',
  'c_acc_net_rem_vol_act',
  'c_press_ap_act',
  'c_press_vp_act',
  'c_press_fp_act',
  'c_press_tmp_act',
])

export function TherapyDetail() {
  const { id } = useParams<{ id: string }>()
  const navigate = useNavigate()
  const [signals, setSignals] = useState<DashboardSignal[]>([])
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    if (!id) return
    patientsApi.getTherapyDashboard(Number(id))
      .then(res => setSignals(res.signals))
      .catch(console.error)
      .finally(() => setLoading(false))
  }, [id])

  const filteredSignals = useMemo(
    () => signals.filter(s => SIGNALS_TO_SHOW.has(s.internal_name)),
    [signals]
  )

  return (
    <div>
      <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 mb-5">
        <div className='flex gap-1'>
          <button onClick={() => navigate(-1)} className="px-3 py-1.5 flex items-center gap-1.5 text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) cursor-pointer">
            <ArrowLeft className="w-4 h-4" />
          </button>
          <h2 className="text-lg md:text-xl font-bold text-(--text-primary)">Terapia #{id}</h2>
        </div>
        <button onClick={() => triggerTherapyExport(Number(id)).catch(console.error)} className="flex items-center gap-1.5 px-3 py-1.5 text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) cursor-pointer">
          <FileDown className="w-4 h-4" /> Exportar
        </button>
      </div>

      {loading ? <Spinner message="Cargando terapia..." /> : (
        filteredSignals.length === 0
          ? <div className="text-center py-10 text-(--text-muted) text-sm">Sin datos de señales para esta terapia</div>
          : filteredSignals.map(s => <Chart key={s.signal_id} signal={s} />)
      )}
    </div>
  )
}
