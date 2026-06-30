import { Sun, Moon, LogOut, Palette, Menu } from 'lucide-react'
import { useAuth } from '../contexts/AuthContext'
import { useTheme } from '../contexts/ThemeContext'
import { useState, useRef, useEffect } from 'react'

interface TopbarProps {
  title: string
  onToggleSidebar: () => void
}

export function Topbar({ title, onToggleSidebar }: TopbarProps) {
  const { user, logout } = useAuth()
  const { theme, toggleTheme, accent, setAccent, accents } = useTheme()
  const [showAccents, setShowAccents] = useState(false)
  const ref = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (ref.current && !ref.current.contains(e.target as Node)) setShowAccents(false)
    }
    document.addEventListener('mousedown', handler)
    return () => document.removeEventListener('mousedown', handler)
  }, [])

  return (
    <header className="h-[var(--header-height)] px-3 md:px-8 flex items-center justify-between sticky top-0 z-30 bg-[var(--topbar-bg)] backdrop-blur-[12px] border-b border-(--glass-border)">
      <div className="flex items-center gap-3 min-w-0">
        <button onClick={onToggleSidebar} className="md:hidden p-1.5 rounded-sm hover:bg-[var(--surface-hover)] cursor-pointer text-(--text-secondary)">
          <Menu className="w-5 h-5" />
        </button>
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
            <div className="absolute right-0 top-full mt-1 glass p-2 flex gap-1.5 z-50">
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
              <label className="relative w-6 h-6 rounded-full cursor-pointer border-2 border-dashed border-(--text-muted) hover:border-(--text-secondary) flex items-center justify-center overflow-hidden">
                <span className="text-(--text-muted) text-xs leading-none">+</span>
                <input
                  type="color"
                  value={accent}
                  onChange={e => { setAccent(e.target.value); setShowAccents(false) }}
                  className="absolute inset-0 opacity-0 cursor-pointer w-full h-full"
                />
              </label>
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
