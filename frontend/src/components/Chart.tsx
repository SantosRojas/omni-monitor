import { useEffect, useMemo, useRef, useState } from 'react'
import {
  LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip,
  ResponsiveContainer, Legend, ReferenceArea,
} from 'recharts'
import { Expand, Minimize } from 'lucide-react'
import type { DashboardSignal } from '../types'

interface ChartProps {
  signal: DashboardSignal
}

export function Chart({ signal }: ChartProps) {
  const wrapRef = useRef<HTMLDivElement>(null)
  const chartRef = useRef<HTMLDivElement>(null)
  const [zoomDomain, setZoomDomain] = useState<{ start: number; end: number } | null>(null)
  const [isDragging, setIsDragging] = useState(false)
  const [dragRefArea, setDragRefArea] = useState<{ x1: number; x2: number } | null>(null)
  const [isFullscreen, setIsFullscreen] = useState(false)

  const signalName = signal.display_name || signal.internal_name

  const data = useMemo(() =>
    signal.values.map(v => ({
      time: new Date(v.timestamp + 'Z').getTime(),
      value: v.value,
    })),
    [signal]
  )

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

  const formatTick = (ms: number) =>
    new Date(ms).toLocaleString('es-PE', {
      month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit',
    })

  if (data.length === 0) return null

  return (
    <div
      ref={wrapRef}
      className={`glass p-4 mb-4 ${isFullscreen ? '!fixed inset-0 z-[9999] flex flex-col' : ''}`}
    >
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-base font-semibold text-(--text-primary)">
          {signalName}
          {signal.unit && <span className="text-(--text-muted) text-sm ml-2">({signal.unit})</span>}
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
          <LineChart data={data}>
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
              formatter={(val: number) => [val.toFixed(2), signalName]}
              contentStyle={{
                background: 'var(--glass-bg)',
                border: '1px solid var(--glass-border)',
                borderRadius: '8px',
                color: 'var(--text-primary)',
              }}
            />
            <Legend formatter={() => signalName} />
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
            <Line
              type="monotone"
              dataKey="value"
              name={signalName}
              stroke="var(--accent)"
              strokeWidth={2}
              dot={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>

      {!isFullscreen && (
        <div className="flex gap-4 mt-2 text-xs text-(--text-muted)">
          <span>Prom: {signal.average?.toFixed(2) ?? '-'}</span>
          <span>Mín: {signal.minimum?.toFixed(2) ?? '-'}</span>
          <span>Máx: {signal.maximum?.toFixed(2) ?? '-'}</span>
          <span>Lecturas: {signal.count}</span>
        </div>
      )}
    </div>
  )
}
