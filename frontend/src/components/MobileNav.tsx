import { NavLink, useLocation } from 'react-router-dom'
import { Users, HardDrive, UserCog, GitCompare, Zap } from 'lucide-react'
import { useAuth } from '../contexts/AuthContext'

const links = [
  { to: '/patients', label: 'Pacientes', icon: Users },
  { to: '/admin/machine-ips', label: 'IPs', icon: HardDrive },
  { to: '/admin/users', label: 'Usuarios', icon: UserCog, adminOnly: true },
  { to: '/admin/equivalences', label: 'Equivalencias', icon: GitCompare, adminOnly: true },
  { to: '/admin/signals', label: 'Señales', icon: Zap, adminOnly: true },
]

export function MobileNav() {
  const { isAdmin } = useAuth()
  const location = useLocation()

  const isActive = (to: string) =>
    location.pathname === to || location.pathname.startsWith(to + '/')

  const visibleLinks = links.filter(l => !l.adminOnly || isAdmin)

  return (
    <nav className="fixed bottom-0 left-0 right-0 z-50 md:hidden flex items-center justify-around px-2 h-16 bg-[var(--sidebar-bg-mobile)] backdrop-blur-[12px] border-t border-(--glass-border) pb-[env(safe-area-inset-bottom,0px)]">
      {visibleLinks.map(l => {
        const active = isActive(l.to)
        return (
          <NavLink
            key={l.to}
            to={l.to}
            end
            className={`relative flex flex-col items-center gap-0.5 min-w-0 px-3 py-1.5 rounded-sm no-underline transition-all duration-200
              ${active
                ? '!text-[var(--accent)]'
                : 'text-(--text-muted) hover:text-(--text-secondary)'
              }`}
          >
            {active && (
              <span className="absolute -top-1 left-1/2 -translate-x-1/2 w-8 h-0.5 rounded-full bg-[var(--accent)]" />
            )}
            <l.icon className={`w-5 h-5 transition-transform duration-200 ${active ? 'scale-110' : ''}`} />
            <span className="text-[10px] font-medium leading-tight truncate max-w-full">
              {l.label}
            </span>
          </NavLink>
        )
      })}
    </nav>
  )
}
