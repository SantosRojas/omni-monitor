import { useEffect, useMemo, useRef, useState } from 'react'
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip,
  ResponsiveContainer, ReferenceArea,
} from 'recharts'
import { Expand, Minimize } from 'lucide-react'
import type { DashboardSignal } from '../types'

interface ChartProps {
  title: string
  signals: DashboardSignal[]
}

const SIGNAL_COLORS: Record<string, string> = {
  'c_press_ap_act': '#ef4444',
  'c_press_vp_act': '#3b82f6',
  'c_press_fp_act': '#22c55e',
  'c_press_tmp_act': '#f59e0b',
  'c_press_ep_act': '#a855f7',
  'c_pump_bs_bl_flow_act': '#ef4444',
  'c_net_rem_flow_act': '#3b82f6',
  'c_pump_fs_mid_flow_act': '#22c55e',
  'd_renal_dose_act': '#f59e0b',
  'c_acc_net_rem_vol_act': '#a855f7',
  'g_patient_data_weight_set': '#ec4899',
}

const FALLBACK_COLORS = ['#ef4444', '#3b82f6', '#22c55e', '#f59e0b', '#a855f7', '#ec4899', '#14b8a6', '#f97316', '#06b6d4', '#84cc16']

function getColor(internalName: string, index: number): string {
  return SIGNAL_COLORS[internalName] || FALLBACK_COLORS[index % FALLBACK_COLORS.length]
}

export function Chart({ title, signals }: ChartProps) {
  const wrapRef = useRef<HTMLDivElement>(null)
  const chartRef = useRef<HTMLDivElement>(null)
  const [zoomDomain, setZoomDomain] = useState<{ start: number; end: number } | null>(null)
  const [isDragging, setIsDragging] = useState(false)
  const [dragRefArea, setDragRefArea] = useState<{ x1: number; x2: number } | null>(null)
  const [isFullscreen, setIsFullscreen] = useState(false)
  const [hiddenSignals, setHiddenSignals] = useState<Set<string>>(new Set())

  const data = useMemo(() => {
    const timeMap = new Map<number, Record<string, number>>()
    for (const s of signals) {
      for (const v of s.values) {
        const ts = v.timestamp.endsWith('Z') ? v.timestamp : v.timestamp + 'Z'
        const ms = new Date(ts).getTime()
        const minute = Math.floor(ms / 60000) * 60000
        if (!timeMap.has(minute)) timeMap.set(minute, {})
        timeMap.get(minute)![s.internal_name] = v.value
      }
    }
    const sorted = [...timeMap.keys()].sort((a, b) => a - b)
    return sorted.map(t => ({ time: t, ...timeMap.get(t)! }))
  }, [signals])

  const filteredData = useMemo(() => {
    if (hiddenSignals.size === 0) return data
    return data.map(d => {
      const point: { time: number; [key: string]: number | null } = { ...d }
      for (const name of hiddenSignals) point[name] = null
      return point
    })
  }, [data, hiddenSignals])

  const dataMin = data.length > 0 ? data[0].time : 0
  const dataMax = data.length > 0 ? data[data.length - 1].time : 0

  const currentDomain = useMemo<[number, number]>(
    () => zoomDomain ? [zoomDomain.start, zoomDomain.end] : [dataMin, dataMax],
    [zoomDomain, dataMin, dataMax]
  )

  const dragState = useRef<{ left: number | null; right: number | null; active: boolean }>({
    left: null, right: null, active: false,
  })

  const getTimeFromMouse = (clientX: number) => {
    const el = chartRef.current
    if (!el) return 0
    const rect = el.getBoundingClientRect()
    const ratio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width))
    return currentDomain[0] + ratio * (currentDomain[1] - currentDomain[0])
  }

  const handleMouseDown = (e: React.MouseEvent) => {
    if (data.length < 2) return
    const t = getTimeFromMouse(e.clientX)
    dragState.current = { left: t, right: t, active: true }
    setIsDragging(true)
  }

  const handleMouseMove = (e: React.MouseEvent) => {
    if (!dragState.current.active) return
    const t = getTimeFromMouse(e.clientX)
    dragState.current.right = t
    setDragRefArea({
      x1: Math.min(dragState.current.left!, t),
      x2: Math.max(dragState.current.left!, t),
    })
  }

  const handleMouseUp = () => {
    if (!dragState.current.active) return
    const { left, right } = dragState.current
    dragState.current.active = false
    setIsDragging(false)
    setDragRefArea(null)
    if (left !== null && right !== null) {
      const start = Math.min(left, right)
      const end = Math.max(left, right)
      if (end - start > 60000) setZoomDomain({ start, end })
    }
  }

  const resetZoom = () => setZoomDomain(null)

  const toggleFullscreen = async () => {
    if (!wrapRef.current) return
    if (document.fullscreenElement) {
      await document.exitFullscreen()
    } else {
      await wrapRef.current.requestFullscreen()
    }
  }

  useEffect(() => {
    const handler = () => setIsFullscreen(!!document.fullscreenElement)
    document.addEventListener('fullscreenchange', handler)
    return () => document.removeEventListener('fullscreenchange', handler)
  }, [])

  const toggleSignal = (internalName: string) => {
    setHiddenSignals(prev => {
      const next = new Set(prev)
      if (next.has(internalName)) {
        next.delete(internalName)
      } else {
        next.add(internalName)
      }
      return next
    })
  }

  const formatTick = (ms: number) =>
    new Date(ms).toLocaleString('es-PE', {
      month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
    })

  if (data.length === 0 || signals.length === 0) return null

  return (
    <div
      ref={wrapRef}
      className={`glass p-4 mb-4 ${isFullscreen ? 'fixed! inset-0 z-9999 flex flex-col' : ''}`}
    >
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-base font-semibold text-(--text-primary)">
          {title}
        </h3>

        <div className="flex items-center gap-1">
          {zoomDomain && (
            <button
              onClick={resetZoom}
              className="px-2 py-0.5 text-xs rounded-sm border border-(--border-subtle) bg-(--surface-btn) hover:bg-(--surface-btn-hover) text-(--text-secondary) cursor-pointer"
            >
              Reset zoom
            </button>
          )}
          <button
            onClick={toggleFullscreen}
            className="p-1 rounded-sm hover:bg-(--surface-hover) cursor-pointer text-(--text-secondary)"
          >
            {isFullscreen ? <Minimize className="w-4 h-4" /> : <Expand className="w-4 h-4" />}
          </button>
        </div>
      </div>

      <div
        ref={chartRef}
        className={isFullscreen ? 'flex-1 select-none' : 'h-48 md:h-64 select-none'}
        style={{ cursor: isDragging ? 'grabbing' : data.length > 1 ? 'crosshair' : 'default' }}
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={filteredData}>
            <CartesianGrid strokeDasharray="3 3" stroke="rgba(255,255,255,0.08)" />
            <XAxis
              dataKey="time"
              type="number"
              scale="time"
              domain={currentDomain}
              tickFormatter={formatTick}
              tick={{ fontSize: 11, fill: 'var(--text-muted)' }}
            />
            <YAxis tick={{ fontSize: 11, fill: 'var(--text-muted)' }} />
            <Tooltip
              labelFormatter={ms => new Date(ms).toLocaleString('es-PE')}
              formatter={(value: number, name: string) => [value.toFixed(2), name]}
              contentStyle={{
                background: 'var(--sidebar-bg)',
                border: '1px solid var(--glass-border)',
                borderRadius: '8px',
                color: 'var(--text-primary)',
              }}
            />
            {dragRefArea && isDragging && (
              <ReferenceArea
                x1={dragRefArea.x1}
                x2={dragRefArea.x2}
                fill="var(--accent)"
                fillOpacity={0.1}
                stroke="var(--accent)"
                strokeOpacity={0.3}
              />
            )}
            {signals.map(s => {
              const color = getColor(s.internal_name, signals.findIndex(x => x.internal_name === s.internal_name))
              return (
                <Line
                  key={s.internal_name}
                  type="monotone"
                  dataKey={(d: Record<string, number | null>) => d?.[s.internal_name]}
                  name={`${s.display_name || s.internal_name}${s.unit ? ` (${s.unit})` : ''}`}
                  stroke={color}
                  strokeWidth={2}
                  dot={false}
                  connectNulls={true}
                />
              )
            })}
          </LineChart>
        </ResponsiveContainer>
      </div>

      <div className="flex flex-wrap gap-x-4 gap-y-1 justify-center text-xs mt-2" style={{ padding: 0, listStyle: 'none' }}>
          {signals.map((s, i) => {
            const hidden = hiddenSignals.has(s.internal_name)
            const color = getColor(s.internal_name, i)
            return (
              <span
                key={s.internal_name}
                onClick={() => toggleSignal(s.internal_name)}
                className="flex items-center gap-1.5"
                style={{
                  cursor: 'pointer',
                  opacity: hidden ? 0.4 : 1,
                  textDecoration: hidden ? 'line-through' : 'none',
                }}
              >
                <span style={{ width: 8, height: 8, borderRadius: '50%', backgroundColor: color, display: 'inline-block', flexShrink: 0 }} />
                {s.display_name || s.internal_name}
              </span>
            )
          })}
        </div>

      {(() => {
        const visibleStats = signals.filter(s => !hiddenSignals.has(s.internal_name))
        if (visibleStats.length === 0) return null
        return (
          <div className="flex flex-wrap gap-x-4 gap-y-1 mt-2 text-xs text-(--text-muted)">
            {visibleStats.map(s => (
              <span key={s.internal_name}>
                {s.display_name || s.internal_name}: Prom {s.average?.toFixed(2) ?? '-'} |
                Mín {s.minimum?.toFixed(2) ?? '-'} | Máx {s.maximum?.toFixed(2) ?? '-'}
              </span>
            ))}
          </div>
        )
      })()}
    </div>
  )
}
