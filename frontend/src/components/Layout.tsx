import { useState } from 'react'
import { Outlet, useLocation } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { Topbar } from './Topbar'

const titles: Record<string, string> = {
  '/patients': 'Pacientes',
  '/admin/machine-ips': 'IPs de Máquinas',
  '/admin/users': 'Usuarios',
}

export function Layout() {
  const [sidebarOpen, setSidebarOpen] = useState(false)
  const location = useLocation()
  const base = '/' + location.pathname.split('/').filter(Boolean)[0]
  const title = titles[base] || 'Monitor OMNI'

  return (
    <div className="flex min-h-screen">
      <Sidebar open={sidebarOpen} onClose={() => setSidebarOpen(false)} />
      <div className="flex-1 flex flex-col min-h-screen md:ml-[var(--sidebar-width)]">
        <Topbar title={title} onToggleSidebar={() => setSidebarOpen(prev => !prev)} />
        <main className="flex-1 p-4 md:p-8">
          <Outlet />
        </main>
      </div>
    </div>
  )
}
