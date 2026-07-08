import { useEffect } from 'react'
import { X, CheckCircle, AlertCircle, Info } from 'lucide-react'

export type ToastType = 'success' | 'error' | 'info'

interface ToastProps {
  id: number
  message: string
  type: ToastType
  onDismiss: (id: number) => void
}

const icons: Record<ToastType, React.ReactNode> = {
  success: <CheckCircle className="w-4 h-4 text-green-400 shrink-0" />,
  error: <AlertCircle className="w-4 h-4 text-red-400 shrink-0" />,
  info: <Info className="w-4 h-4 text-blue-400 shrink-0" />,
}

const borders: Record<ToastType, string> = {
  success: 'border-green-500/40',
  error: 'border-red-500/40',
  info: 'border-blue-500/40',
}

export function Toast({ id, message, type, onDismiss }: ToastProps) {
  useEffect(() => {
    const timer = setTimeout(() => onDismiss(id), 4000)
    return () => clearTimeout(timer)
  }, [id, onDismiss])

  return (
      <div
        className={`flex items-start gap-2.5 px-4 py-3 rounded-sm border ${borders[type]} bg-(--bg-primary) shadow-lg text-sm text-(--text-primary) min-w-[280px] max-w-[420px] animate-slide-up`}
      >
      {icons[type]}
      <span className="flex-1">{message}</span>
      <button onClick={() => onDismiss(id)} className="text-(--text-muted) hover:text-(--text-primary) cursor-pointer shrink-0">
        <X className="w-4 h-4" />
      </button>
    </div>
  )
}
