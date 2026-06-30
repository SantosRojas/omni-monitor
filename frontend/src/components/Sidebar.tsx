import { NavLink } from 'react-router-dom'
import { Users, HardDrive, UserCog, Activity, X } from 'lucide-react'
import { useAuth } from '../contexts/AuthContext'

const links = [
  { to: '/patients', label: 'Pacientes', icon: Users },
  { to: '/admin/machine-ips', label: 'IPs de Máquinas', icon: HardDrive },
  { to: '/admin/users', label: 'Usuarios', icon: UserCog, adminOnly: true },
]

interface SidebarProps {
  open: boolean
  onClose: () => void
}

export function Sidebar({ open, onClose }: SidebarProps) {
  const { isAdmin, user } = useAuth()

  const sidebarContent = (
    <>
      <div className="flex items-center justify-between px-4 py-3 mb-4">
        <NavLink to="/patients" onClick={onClose} className="flex items-center gap-3 text-lg font-bold text-(--text-primary) no-underline">
          <Activity className="w-6 h-6 text-[var(--accent)]" />
          <span>Monitor <span className="text-[var(--accent)]">OMNI</span></span>
        </NavLink>
        <button onClick={onClose} className="md:hidden p-1 text-(--text-muted) hover:text-(--text-primary) cursor-pointer">
          <X className="w-5 h-5" />
        </button>
      </div>

      <nav className="flex flex-col gap-1 flex-1 px-2">
        {links.map(l => {
          if (l.adminOnly && !isAdmin) return null
          return (
            <NavLink
              key={l.to}
              to={l.to}
              end
              onClick={onClose}
              className={({ isActive }) =>
                `flex items-center gap-3 px-4 py-2.5 rounded-sm text-sm text-(--text-secondary) no-underline transition-all duration-200 hover:bg-[var(--surface-hover)] hover:text-(--text-primary) ${isActive ? '!bg-[var(--accent)]/20 !text-[var(--accent)] border border-[var(--accent)]/30' : ''
                }`
              }
            >
              <l.icon className="w-4 h-4 shrink-0" />
              <span>{l.label}</span>
            </NavLink>
          )
        })}
      </nav>

      {user && (
        <div className="px-4 py-3 border-t border-(--glass-border) mt-auto">
          <div className="flex items-center gap-3 text-sm text-(--text-secondary)">
            <div className="w-8 h-8 rounded-full bg-[var(--accent)]/20 flex items-center justify-center text-[var(--accent)] font-semibold shrink-0">
              {user.full_name.charAt(0).toUpperCase()}
            </div>
            <div className="min-w-0">
              <div className="truncate font-medium text-(--text-primary)">{user.full_name}</div>
              <div className="truncate text-xs text-(--text-muted)">{user.role}</div>
            </div>
          </div>
        </div>
      )}
    </>
  )

  return (
    <>
      <aside className="hidden md:flex fixed left-0 top-0 z-40 h-screen w-[var(--sidebar-width)] flex-col gap-2 px-4 py-5 border-r border-(--glass-border) bg-[var(--sidebar-bg)] backdrop-blur-[12px]">
        {sidebarContent}
      </aside>

      {open && (
        <div className="fixed inset-0 z-50 md:hidden" onClick={onClose}>
          <div className="absolute inset-0 bg-black/50 backdrop-blur-sm" />
          <aside
            className="relative w-[var(--sidebar-width)] h-screen flex flex-col gap-2 px-4 py-5 border-r border-(--glass-border) bg-[var(--sidebar-bg-mobile)] backdrop-blur-[12px] animate-slide-in"
            onClick={e => e.stopPropagation()}
          >
            {sidebarContent}
          </aside>
        </div>
      )}
    </>
  )
}
