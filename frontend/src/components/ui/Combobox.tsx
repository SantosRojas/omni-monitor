import { useState, useRef, useEffect, useMemo } from 'react'
import { ChevronDown } from 'lucide-react'
import { Input } from './Input'

export interface ComboboxOption {
  value: number
  label: string
}

interface ComboboxProps {
  options: ComboboxOption[]
  value: number
  onChange: (value: number, label: string) => void
  placeholder?: string
}

export function Combobox({ options, value, onChange, placeholder = 'Seleccionar…' }: ComboboxProps) {
  const [open, setOpen] = useState(false)
  const [query, setQuery] = useState('')
  const ref = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLInputElement>(null)

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) {
        setOpen(false)
        const opt = options.find(o => o.value === value)
        if (opt) setQuery(opt.label)
      }
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [options, value])

  const filtered = useMemo(() => {
    if (!query) return options
    const lower = query.toLowerCase()
    return options.filter(o => o.label.toLowerCase().includes(lower))
  }, [options, query])

  const selected = options.find(o => o.value === value)
  const isNew = value === 0 && query.length > 0 && !filtered.some(o => o.label.toLowerCase() === query.toLowerCase())

  const handleSelect = (opt: ComboboxOption) => {
    setQuery(opt.label)
    onChange(opt.value, opt.label)
    setOpen(false)
  }

  const handleCreate = () => {
    onChange(0, query)
    setOpen(false)
  }

  const handleFocus = () => {
    setOpen(true)
    if (selected && !query) {
      setQuery(selected.label)
    }
  }

  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = e.target.value
    setQuery(val)
    if (!open) setOpen(true)
    const exact = options.find(o => o.label.toLowerCase() === val.toLowerCase())
    if (exact) {
      onChange(exact.value, exact.label)
    } else {
      onChange(0, val)
    }
  }

  return (
    <div className="relative" ref={ref}>
      <div className="relative">
        <Input
          ref={inputRef}
          value={query}
          onChange={handleInputChange}
          onFocus={handleFocus}
          placeholder={placeholder}
        />
        <button
          type="button"
          tabIndex={-1}
          onClick={() => { setOpen(!open); if (!open) inputRef.current?.focus() }}
          className="absolute right-2 top-1/2 -translate-y-1/2 cursor-pointer"
        >
          <ChevronDown className={`w-4 h-4 text-(--text-muted) transition-transform ${open ? 'rotate-180' : ''}`} />
        </button>
      </div>

      {open && (
        <div
          className="absolute z-50 mt-1 w-full max-h-60 overflow-y-auto rounded-sm"
          style={{
            background: 'var(--dropdown-bg)',
            border: '1px solid var(--glass-border)',
          }}
        >
          {filtered.map(opt => (
            <button
              key={opt.value}
              type="button"
              onClick={() => handleSelect(opt)}
              className="w-full text-left px-3 py-2 text-sm transition-colors cursor-pointer"
              style={{
                color: opt.value === value ? 'var(--accent)' : 'var(--text-secondary)',
                fontWeight: opt.value === value ? 600 : 400,
                background: opt.value === value ? 'color-mix(in srgb, var(--accent) 18%, transparent)' : 'transparent',
              }}
              onMouseEnter={e => { if (opt.value !== value) e.currentTarget.style.background = 'var(--dropdown-hover)' }}
              onMouseLeave={e => { if (opt.value !== value) e.currentTarget.style.background = 'transparent' }}
            >
              {opt.label}
            </button>
          ))}
          {isNew && (
            <button
              type="button"
              onClick={handleCreate}
              className="w-full text-left px-3 py-2 text-sm cursor-pointer border-t"
              style={{
                color: 'var(--accent)',
                borderColor: 'var(--glass-border)',
                fontWeight: 500,
              }}
              onMouseEnter={e => { e.currentTarget.style.background = 'var(--dropdown-hover)' }}
              onMouseLeave={e => { e.currentTarget.style.background = 'transparent' }}
            >
              + Agregar &quot;{query}&quot;
            </button>
          )}
          {!filtered.length && !isNew && (
            <div className="px-3 py-2 text-sm text-(--text-muted)">Sin resultados</div>
          )}
        </div>
      )}
    </div>
  )
}
