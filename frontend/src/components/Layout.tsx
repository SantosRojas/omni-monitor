import { useLocation } from 'react-router-dom'
import { Outlet } from 'react-router-dom'
import { Sidebar } from './Sidebar'
import { Topbar } from './Topbar'
import { MobileNav } from './MobileNav'

const titles: Record<string, string> = {
  '/patients': 'Pacientes',
  '/admin/machine-ips': 'IPs de Máquinas',
  '/admin/users': 'Usuarios',
}

export function Layout() {
  const location = useLocation()
  const base = '/' + location.pathname.split('/').filter(Boolean)[0]
  const title = titles[base] || 'Monitor OMNI'

  return (
    <div className="flex min-h-screen">
      <Sidebar />
      <div className="flex-1 flex flex-col min-h-screen">
        <Topbar title={title} />
        <main className="flex-1 p-4 md:p-8 pb-20 md:pb-8">
          <Outlet />
        </main>
      </div>
      <MobileNav />
    </div>
  )
}
