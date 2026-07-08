import { useState } from 'react'
import { NavLink, useLocation, useNavigate } from 'react-router-dom'
import {
  Users, HardDrive, UserCog, Activity, GitCompare, Zap,
  ChevronsLeft, ChevronsRight,
  Sun, Moon,
  LogOut,
} from 'lucide-react'
import { useAuth } from '../contexts/AuthContext'
import { useTheme } from '../contexts/ThemeContext'

const navItems = [
  { to: '/patients', label: 'Pacientes', icon: Users },
]

const adminNavItems = [
  { to: '/admin/machine-ips', label: 'IPs de Máquinas', icon: HardDrive },
  { to: '/admin/users', label: 'Usuarios', icon: UserCog, adminOnly: true },
  { to: '/admin/equivalences', label: 'Equivalencias', icon: GitCompare, adminOnly: true },
  { to: '/admin/signals', label: 'Señales', icon: Zap, adminOnly: true },
]

export function Sidebar() {
  const [collapsed, setCollapsed] = useState(() => localStorage.getItem('sidebar_collapsed') === 'true')
  const { isAdmin, user, logout } = useAuth()
  const { theme, toggleTheme } = useTheme()
  const location = useLocation()
  const navigate = useNavigate()

  function toggleCollapsed() {
    setCollapsed(prev => {
      const next = !prev
      localStorage.setItem('sidebar_collapsed', String(next))
      return next
    })
  }

  const isActive = (to: string) =>
    location.pathname === to || location.pathname.startsWith(to + '/')

  function handleLogout() {
    logout()
    navigate('/login')
  }

  const filteredAdminItems = adminNavItems.filter(l => !l.adminOnly || isAdmin)

  function linkClass(to: string) {
    const active = isActive(to)
    return `flex items-center rounded-lg px-3 py-2 text-sm transition-colors no-underline
      ${collapsed ? 'justify-center px-0 mx-auto w-10 h-10' : 'gap-3'}
      ${active
        ? 'bg-(--glass-bg) backdrop-blur-[12px] text-(--accent) font-medium shadow-sm border border-[var(--accent)]/30'
        : 'text-(--text-secondary) hover:glass-hover hover:text-(--text-primary)'}`
  }

  const sidebarContent = (
    <div className="flex h-full flex-col gap-4">
      <div className={`flex items-center ${collapsed ? 'justify-center px-0' : 'px-4'} py-4`}>
        <NavLink
          to="/patients"
          className="flex items-center gap-3 text-lg font-bold text-(--text-primary) no-underline"
          title={collapsed ? 'Monitor OMNI' : undefined}
        >
          <div className="flex h-8 w-8 items-center justify-center rounded-lg bg-(--accent) shadow-sm">
            <Activity className="h-4 w-4 text-white" />
          </div>
          {!collapsed && (
            <span className="font-semibold tracking-tight">Monitor <span className="text-(--accent)">OMNI</span></span>
          )}
        </NavLink>
      </div>

      <nav className={`flex-1 overflow-y-auto sidebar-scroll ${collapsed ? 'px-2' : 'px-3'}`}>
        {navItems.map(item => (
          <NavLink
            key={item.to}
            to={item.to}
            end={item.to === '/patients'}
            className={linkClass(item.to)}
            title={collapsed ? item.label : undefined}
          >
            <item.icon className="h-4 w-4 shrink-0" />
            {!collapsed && item.label}
          </NavLink>
        ))}

        {filteredAdminItems.length > 0 && (
          <>
            <div className="my-2 border-t border-(--glass-border)" />
            {!collapsed && (
              <p className="px-3 py-1.5 text-[11px] font-semibold uppercase tracking-widest text-(--text-muted)">
                Administración
              </p>
            )}
            {filteredAdminItems.map(item => (
              <NavLink
                key={item.to}
                to={item.to}
                className={linkClass(item.to)}
                title={collapsed ? item.label : undefined}
              >
                <item.icon className="h-4 w-4 shrink-0" />
                {!collapsed && item.label}
              </NavLink>
            ))}
          </>
        )}
      </nav>

      <div className="border-t border-(--glass-border) p-3">
        <div className={`flex items-center ${collapsed ? 'justify-center' : 'gap-3 px-3'} mb-3`}>
          <div className="flex h-8 w-8 shrink-0 items-center justify-center rounded-full bg-(--accent)/20 text-sm font-medium text-(--accent) ring-2 ring-[var(--accent)]/20">
            {user?.full_name?.charAt(0)?.toUpperCase() || user?.username?.charAt(0)?.toUpperCase() || 'U'}
          </div>
          {!collapsed && (
            <div className="flex-1 truncate">
              <p className="text-sm text-(--text-primary) font-medium">{user?.full_name || user?.username}</p>
              <p className="text-xs text-(--text-muted) capitalize">{user?.role}</p>
            </div>
          )}
        </div>

        {!collapsed ? (
          <div className="mb-2 flex rounded-lg border border-(--glass-border) bg-(--glass-bg) p-0.5">
            <button
              onClick={() => theme !== 'light' && toggleTheme()}
              className={`flex flex-1 items-center justify-center rounded-md px-2 py-1.5 text-xs transition-all ${theme === 'light' ? 'bg-(--surface-card) text-(--text-primary) shadow-sm' : 'text-(--text-muted) hover:text-(--text-primary)'}`}
              title="Claro"
            >
              <Sun className="h-3.5 w-3.5" />
            </button>
            <button
              onClick={() => theme !== 'dark' && toggleTheme()}
              className={`flex flex-1 items-center justify-center rounded-md px-2 py-1.5 text-xs transition-all ${theme === 'dark' ? 'bg-(--surface-card) text-(--text-primary) shadow-sm' : 'text-(--text-muted) hover:text-(--text-primary)'}`}
              title="Oscuro"
            >
              <Moon className="h-3.5 w-3.5" />
            </button>
          </div>
        ) : (
          <div className="mb-2 flex justify-center">
            <button
              onClick={toggleTheme}
              className="p-1.5 rounded-sm text-(--text-muted) hover:text-(--text-primary) hover:bg-(--surface-hover) cursor-pointer"
              title={theme === 'dark' ? 'Cambiar a claro' : 'Cambiar a oscuro'}
            >
              {theme === 'dark' ? <Sun className="h-4 w-4" /> : <Moon className="h-4 w-4" />}
            </button>
          </div>
        )}

        <button
          onClick={handleLogout}
          className={`flex items-center rounded-lg px-3 py-2 text-sm text-(--text-secondary) hover:text-(--danger) hover:bg-(--surface-hover) transition-colors no-underline w-full ${collapsed ? 'justify-center px-0' : 'gap-3'}`}
          title={collapsed ? 'Cerrar sesión' : undefined}
        >
          <LogOut className="h-4 w-4 shrink-0" />
          {!collapsed && 'Cerrar sesión'}
        </button>
      </div>
    </div>
  )

  return (
    <aside
      className={`relative hidden shrink-0 border-r border-(--glass-border) bg-(--sidebar-bg) backdrop-blur-[12px] transition-all duration-200 md:block ${collapsed ? 'w-16' : 'w-[var(--sidebar-width)]'}`}
    >
      {sidebarContent}

      <button
        onClick={toggleCollapsed}
        className="absolute -right-3 top-1/2 z-10 flex h-6 w-6 -translate-y-1/2 items-center justify-center rounded-full border border-(--glass-border) bg-(--sidebar-bg) text-(--text-muted) shadow-sm hover:text-(--text-primary)"
        title={collapsed ? 'Expandir' : 'Colapsar'}
      >
        {collapsed ? <ChevronsRight className="h-3.5 w-3.5" /> : <ChevronsLeft className="h-3.5 w-3.5" />}
      </button>
    </aside>
  )
}
