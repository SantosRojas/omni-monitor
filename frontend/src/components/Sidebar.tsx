import { useState } from 'react'
import { NavLink, useLocation } from 'react-router-dom'
import { Users, HardDrive, UserCog, Activity, ChevronLeft, ChevronRight } from 'lucide-react'
import { useAuth } from '../contexts/AuthContext'

const sections = [
  {
    label: 'Principal',
    links: [
      { to: '/patients', label: 'Pacientes', icon: Users },
    ],
  },
  {
    label: 'Administración',
    links: [
      { to: '/admin/machine-ips', label: 'IPs de Máquinas', icon: HardDrive },
      { to: '/admin/users', label: 'Usuarios', icon: UserCog, adminOnly: true },
    ],
  },
]

interface SidebarProps {
  collapsed: boolean
  onToggleCollapse: () => void
}

export function Sidebar({ collapsed, onToggleCollapse }: SidebarProps) {
  const { isAdmin, user } = useAuth()
  const location = useLocation()
  const [hovered, setHovered] = useState(false)

  const expanded = !collapsed || hovered

  const isActive = (to: string) =>
    location.pathname === to || location.pathname.startsWith(to + '/')

  const filteredSections = sections.map(s => ({
    ...s,
    links: s.links.filter(l => !l.adminOnly || isAdmin),
  })).filter(s => s.links.length > 0)

  return (
    <aside
      onMouseEnter={() => setHovered(true)}
      onMouseLeave={() => setHovered(false)}
      className={`sidebar-width hidden md:flex fixed left-0 top-0 z-40 h-screen flex-col border-r border-(--glass-border) bg-[var(--sidebar-bg)] backdrop-blur-[12px]
        ${expanded ? 'w-[var(--sidebar-width)]' : 'w-16'}`}
    >
      {/* Brand */}
      <div className={`flex items-center ${expanded ? 'px-4' : 'justify-center'} py-3 mb-4`}>
        <NavLink
          to="/patients"
          className="flex items-center gap-3 text-lg font-bold text-(--text-primary) no-underline"
          title={expanded ? undefined : 'Monitor OMNI'}
        >
          <Activity className="w-6 h-6 text-[var(--accent)] shrink-0" />
          {expanded && (
            <span>Monitor <span className="text-[var(--accent)]">OMNI</span></span>
          )}
        </NavLink>
      </div>

      {/* Navigation */}
      <nav className="flex flex-col flex-1 overflow-y-auto sidebar-scroll px-2">
        {filteredSections.map(section => (
          <div key={section.label} className="mb-3">
            {expanded && (
              <div className="px-4 py-1.5">
                <span className="text-[11px] font-semibold uppercase tracking-widest text-(--text-muted)">
                  {section.label}
                </span>
              </div>
            )}
            <div className="flex flex-col gap-0.5">
              {section.links.map(l => {
                const active = isActive(l.to)
                return (
                  <div key={l.to} className="relative group">
                    <NavLink
                      to={l.to}
                      end
                      className={`nav-link-active flex items-center gap-3 px-4 py-2.5 rounded-sm text-sm no-underline transition-all duration-200
                        ${expanded ? 'px-4' : 'justify-center px-0 mx-auto w-10 h-10'}
                        ${active
                          ? '!text-[var(--accent)] bg-[var(--accent)]/12 border border-[var(--accent)]/25 shadow-[0_0_12px_-4px_var(--accent)]'
                          : 'text-(--text-secondary) hover:bg-[var(--surface-hover)] hover:text-(--text-primary)'
                        }`}
                      title={!expanded ? l.label : undefined}
                    >
                      <l.icon className={`shrink-0 transition-transform duration-200 group-hover:scale-110 ${active ? 'scale-110' : ''} ${expanded ? 'w-4 h-4' : 'w-5 h-5'}`} />
                      {expanded && <span>{l.label}</span>}
                      {active && <span className="nav-active-bar" />}
                    </NavLink>
                    {!expanded && (
                      <span className="sidebar-tooltip">{l.label}</span>
                    )}
                  </div>
                )
              })}
            </div>
          </div>
        ))}
      </nav>

      {/* User card */}
      {user && (
        <div className={`border-t border-(--glass-border) ${expanded ? 'px-4 py-3' : 'px-2 py-3 flex justify-center'}`}>
          {expanded ? (
            <div className="glass-sm flex items-center gap-3 px-3 py-2.5 text-sm text-(--text-secondary)">
              <div className="w-8 h-8 rounded-full bg-gradient-to-br from-[var(--accent)] to-[var(--accent)]/60 flex items-center justify-center text-white font-semibold text-xs shrink-0 shadow-sm">
                {user.full_name.charAt(0).toUpperCase()}
              </div>
              <div className="min-w-0 flex-1">
                <div className="truncate font-medium text-(--text-primary) text-xs leading-tight">{user.full_name}</div>
                <div className="truncate text-[11px] text-(--text-muted) mt-0.5">{user.role}</div>
              </div>
            </div>
          ) : (
            <div className="relative group" title={user.full_name}>
              <div className="w-9 h-9 rounded-full bg-gradient-to-br from-[var(--accent)] to-[var(--accent)]/60 flex items-center justify-center text-white font-semibold text-sm shadow-sm">
                {user.full_name.charAt(0).toUpperCase()}
              </div>
              <span className="sidebar-tooltip">{user.full_name} · {user.role}</span>
            </div>
          )}
        </div>
      )}

      {/* Collapse toggle */}
      <div className={`flex ${expanded ? 'justify-end px-3' : 'justify-center'} py-2`}>
        <button
          onClick={onToggleCollapse}
          className="p-1.5 rounded-sm text-(--text-muted) hover:text-(--text-primary) hover:bg-[var(--surface-hover)] cursor-pointer transition-all duration-200"
          title={collapsed ? 'Expandir sidebar' : 'Colapsar sidebar'}
        >
          {collapsed ? <ChevronRight className="w-4 h-4" /> : <ChevronLeft className="w-4 h-4" />}
        </button>
      </div>
    </aside>
  )
}
