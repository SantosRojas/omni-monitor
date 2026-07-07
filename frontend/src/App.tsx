import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom'
import { AuthProvider, useAuth } from './contexts/AuthContext'
import { ThemeProvider } from './contexts/ThemeContext'
import { Layout } from './components/Layout'
import { LoginPage } from './pages/LoginPage'
import { PatientsPage } from './pages/PatientsPage'
import { PatientDetail } from './pages/PatientDetail'
import { PatientHistory } from './pages/PatientHistory'
import { PatientDashboard } from './pages/PatientDashboard'
import { TherapyDetail } from './pages/TherapyDetail'
import { AdminMachineIps } from './pages/AdminMachineIps'
import { AdminUsers } from './pages/AdminUsers'
import { AdminEquivalences } from './pages/AdminEquivalences'
import { AdminSignals } from './pages/AdminSignals'

function ProtectedRoute({ children }: { children: React.ReactNode }) {
  const { isLoggedIn } = useAuth()
  if (!isLoggedIn) return <Navigate to="/login" replace />
  return <>{children}</>
}

function RootRedirect() {
  const { isLoggedIn } = useAuth()
  return <Navigate to={isLoggedIn ? '/patients' : '/login'} replace />
}

function App() {
  return (
    <BrowserRouter>
      <ThemeProvider>
        <AuthProvider>
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/" element={<RootRedirect />} />
            <Route path="/patients" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
              <Route index element={<PatientsPage />} />
            </Route>
            <Route path="/patients/:id" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
              <Route index element={<PatientDetail />} />
            </Route>
            <Route path="/patients/:id/history" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
              <Route index element={<PatientHistory />} />
            </Route>
            <Route path="/patients/:id/dashboard" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
              <Route index element={<PatientDashboard />} />
            </Route>
            <Route path="/therapies/:id" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
              <Route index element={<TherapyDetail />} />
            </Route>
            <Route path="/admin/machine-ips" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
              <Route index element={<AdminMachineIps />} />
            </Route>
            <Route path="/admin/users" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
              <Route index element={<AdminUsers />} />
            </Route>
            <Route path="/admin/equivalences" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
              <Route index element={<AdminEquivalences />} />
            </Route>
            <Route path="/admin/signals" element={<ProtectedRoute><Layout /></ProtectedRoute>}>
              <Route index element={<AdminSignals />} />
            </Route>
            <Route path="*" element={<Navigate to="/" replace />} />
          </Routes>
        </AuthProvider>
      </ThemeProvider>
    </BrowserRouter>
  )
}

export default App
