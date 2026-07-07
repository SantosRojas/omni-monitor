import { Sun, Moon, LogOut, Palette, Check } from 'lucide-react'
import { useAuth } from '../contexts/AuthContext'
import { useTheme } from '../contexts/ThemeContext'
import { useState, useRef, useEffect } from 'react'

interface TopbarProps {
  title: string
}

export function Topbar({ title }: TopbarProps) {
  const { user, logout } = useAuth()
  const { theme, toggleTheme, accent, setAccent, accents } = useTheme()
  const [showAccents, setShowAccents] = useState(false)
  const [customColor, setCustomColor] = useState(accent)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setShowAccents(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [])

  useEffect(() => {
    setCustomColor(accent)
  }, [accent])

  return (
    <header className="h-[var(--header-height)] px-3 md:px-8 flex items-center justify-between sticky top-0 z-30 bg-[var(--topbar-bg)] backdrop-blur-[12px] border-b border-(--glass-border)">
      <div className="flex items-center gap-3 min-w-0">
        <h1 className="text-sm md:text-lg font-semibold text-(--text-primary) truncate">{title}</h1>
      </div>

      <div className="flex items-center gap-1 md:gap-3 text-sm text-(--text-secondary) shrink-0">
        <div className="relative" ref={ref}>
          <button
            onClick={() => setShowAccents(!showAccents)}
            className="flex items-center gap-1.5 p-1.5 rounded-sm hover:bg-[var(--surface-hover)] cursor-pointer"
          >
            <Palette className="w-4 h-4" style={{ color: accent }} />
          </button>
          {showAccents && (
            <div className="absolute right-0 top-full mt-1 glass p-3 z-50 min-w-[200px]">
              <div className="flex gap-1.5 flex-wrap mb-2">
                {accents.map(a => (
                  <button
                    key={a.value}
                    onClick={() => { setAccent(a.value); setShowAccents(false) }}
                    className={`w-6 h-6 rounded-full cursor-pointer border-2 transition-all ${accent === a.value ? 'border-white scale-110' : 'border-transparent'
                      }`}
                    style={{ background: a.value }}
                    title={a.name}
                  />
                ))}
              </div>
              <div className="flex items-center gap-1.5 pt-2 border-t border-(--border-subtle)">
                <input
                  type="color"
                  value={customColor}
                  onChange={e => setCustomColor(e.target.value)}
                  className="w-7 h-7 rounded cursor-pointer border-0 p-0 shrink-0"
                />
                <input
                  type="text"
                  value={customColor}
                  onChange={e => setCustomColor(e.target.value)}
                  className="flex-1 min-w-0 px-1.5 py-1 text-xs rounded-sm bg-(--surface-btn) border border-(--border-subtle) text-(--text-primary) outline-none"
                  placeholder="#000000"
                />
                <button
                  onClick={() => { setAccent(customColor); setShowAccents(false) }}
                  className="p-1 rounded-sm bg-(--accent) text-white cursor-pointer shrink-0"
                  title="Aceptar color"
                >
                  <Check className="w-3.5 h-3.5" />
                </button>
              </div>
            </div>
          )}
        </div>

        <button onClick={toggleTheme} className="p-1.5 rounded-sm hover:bg-[var(--surface-hover)] cursor-pointer">
          {theme === 'dark' ? <Sun className="w-4 h-4" /> : <Moon className="w-4 h-4" />}
        </button>

        <span className="text-(--text-muted) hidden md:inline">|</span>
        <span className="hidden md:inline truncate max-w-[120px]">{user?.full_name || user?.username}</span>
        <button onClick={logout} className="p-1.5 rounded-sm hover:bg-[var(--surface-hover)] text-[var(--danger)] cursor-pointer">
          <LogOut className="w-4 h-4" />
        </button>
      </div>
    </header>
  )
}
