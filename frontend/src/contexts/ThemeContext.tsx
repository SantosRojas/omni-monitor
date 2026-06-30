import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'

const ACCENTS = [
  { name: 'Púrpura', value: '#6c63ff' },
  { name: 'Azul', value: '#3b82f6' },
  { name: 'Verde', value: '#10b981' },
  { name: 'Ámbar', value: '#f59e0b' },
  { name: 'Rojo', value: '#ef4444' },
  { name: 'Rosa', value: '#ec4899' },
]

interface ThemeContextType {
  theme: 'light' | 'dark'
  accent: string
  accents: typeof ACCENTS
  setTheme: (t: 'light' | 'dark') => void
  setAccent: (c: string) => void
  toggleTheme: () => void
}

const ThemeContext = createContext<ThemeContextType | null>(null)

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<'light' | 'dark'>(() => {
    const saved = localStorage.getItem('monitor_theme')
    if (saved === 'light' || saved === 'dark') return saved
    return 'dark'
  })
  const [accent, setAccentState] = useState(() => {
    return localStorage.getItem('monitor_accent') || '#6c63ff'
  })

  const setTheme = (t: 'light' | 'dark') => {
    setThemeState(t)
    localStorage.setItem('monitor_theme', t)
  }

  const toggleTheme = () => setTheme(theme === 'dark' ? 'light' : 'dark')

  const setAccent = (c: string) => {
    setAccentState(c)
    localStorage.setItem('monitor_accent', c)
  }

  useEffect(() => {
    document.documentElement.classList.toggle('dark', theme === 'dark')
    document.documentElement.style.setProperty('--accent', accent)
    document.documentElement.style.setProperty('--accent-hover', accent + 'cc')

    const isDark = theme === 'dark'
    const g = isDark
      ? `linear-gradient(135deg, #0c0e1a 0%, #1a1a3e 30%, ${accent}70 70%, #0c0e1a 100%)`
      : `linear-gradient(135deg, #e8eef5 0%, #dce4f0 30%, ${accent}60 70%, #e8eef5 100%)`
    document.documentElement.style.setProperty('--bg-gradient', g)
  }, [theme, accent])

  return (
    <ThemeContext.Provider value={{ theme, accent, accents: ACCENTS, setTheme, setAccent, toggleTheme }}>
      {children}
    </ThemeContext.Provider>
  )
}

export function useTheme() {
  const ctx = useContext(ThemeContext)
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider')
  return ctx
}
