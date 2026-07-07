import { useState } from 'react'
import { Outlet, useLocation } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { Topbar } from './Topbar'
import { MobileNav } from './MobileNav'

const titles: Record<string, string> = {
  '/patients': 'Pacientes',
  '/admin/machine-ips': 'IPs de Máquinas',
  '/admin/users': 'Usuarios',
}

export function Layout() {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(() => {
    return localStorage.getItem('sidebar_collapsed') === 'true'
  })
  const location = useLocation()
  const base = '/' + location.pathname.split('/').filter(Boolean)[0]
  const title = titles[base] || 'Monitor OMNI'

  const handleToggleCollapse = () => {
    setSidebarCollapsed(prev => {
      const next = !prev
      localStorage.setItem('sidebar_collapsed', String(next))
      return next
    })
  }

  return (
    <div className="flex min-h-screen">
      <Sidebar
        collapsed={sidebarCollapsed}
        onToggleCollapse={handleToggleCollapse}
      />
      <div className={`flex-1 flex flex-col min-h-screen transition-all duration-300 ${sidebarCollapsed ? 'md:ml-16' : 'md:ml-[var(--sidebar-width)]'}`}>
        <Topbar title={title} />
        <main className="flex-1 p-4 md:p-8 pb-20 md:pb-8">
          <Outlet />
        </main>
      </div>
      <MobileNav />
    </div>
  )
}
