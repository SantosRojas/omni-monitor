import { ChevronDown } from 'lucide-react'
import { useState, useRef, useEffect } from 'react'

export interface SelectOption<T> {
  value: T
  label: string
}

export interface SelectProps<T> {
  options: SelectOption<T>[]
  value: T
  onChange: (value: T) => void
  placeholder?: string
}

export function Select<T extends string | number>({ options, value, onChange, placeholder = 'Seleccionar opción…' }: SelectProps<T>) {
  const [open, setOpen] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setOpen(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [])

  const selected = options.find(o => o.value === value)

  return (
    <div className="relative" ref={ref}>
      <button
        type="button"
        onClick={() => setOpen(!open)}
        className="w-full flex items-center justify-between gap-2 px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none cursor-pointer"
      >
        <span className={selected ? '' : 'text-(--text-muted)'}>
          {selected ? selected.label : placeholder}
        </span>
        <ChevronDown className={`w-4 h-4 text-(--text-muted) transition-transform shrink-0 ${open ? 'rotate-180' : ''}`} />
      </button>

      {open && (
        <div
          className="absolute z-50 mt-1 w-full max-h-60 overflow-y-auto rounded-sm"
          style={{
            background: 'var(--glass-sm-bg)',
            backdropFilter: 'blur(8px)',
            WebkitBackdropFilter: 'blur(8px)',
            border: '1px solid var(--glass-sm-border)',
          }}
        >
          {options.map(opt => (
            <button
              key={String(opt.value)}
              type="button"
              onClick={() => { onChange(opt.value); setOpen(false) }}
              className="w-full text-left px-3 py-2 text-sm transition-colors cursor-pointer"
              style={{
                color: opt.value === value ? 'var(--accent)' : 'var(--text-secondary)',
                fontWeight: opt.value === value ? 600 : 400,
                background: opt.value === value ? 'color-mix(in srgb, var(--accent) 18%, transparent)' : 'transparent',
              }}
              onMouseEnter={e => { if (opt.value !== value) e.currentTarget.style.background = 'var(--surface-hover)' }}
              onMouseLeave={e => { if (opt.value !== value) e.currentTarget.style.background = 'transparent' }}
            >
              {opt.label}
            </button>
          ))}
        </div>
      )}
    </div>
  )
}
