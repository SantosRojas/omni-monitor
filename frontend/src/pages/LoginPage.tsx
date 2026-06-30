import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Activity, LogIn } from 'lucide-react'
import { useAuth } from '../contexts/AuthContext'
import * as authApi from '../api/auth'

export function LoginPage() {
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)
  const { login } = useAuth()
  const navigate = useNavigate()

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)
    try {
      const res = await authApi.login(username, password)
      login(res.token, res.user)
      navigate('/patients')
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Error al iniciar sesión')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="flex items-center justify-center min-h-screen p-6">
      <div className="glass w-full max-w-sm p-10">
        <div className="flex flex-col items-center mb-8">
          <Activity className="w-10 h-10 text-[var(--accent)] mb-2" />
          <h1 className="text-2xl font-bold text-(--text-primary)">Monitor OMNI</h1>
          <p className="text-sm text-(--text-muted)">Inicia sesión para continuar</p>
        </div>

        {error && (
          <div className="bg-red-500/15 border border-red-500/30 text-[var(--danger)] text-sm text-center px-4 py-2.5 rounded-sm mb-4">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div>
            <label className="block mb-1.5 text-xs font-medium text-(--text-secondary)">Usuario</label>
            <input
              className="w-full px-3.5 py-2.5 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)] focus:bg-(--surface-btn) transition-colors"
              value={username}
              onChange={e => setUsername(e.target.value)}
              autoFocus
              required
            />
          </div>
          <div>
            <label className="block mb-1.5 text-xs font-medium text-(--text-secondary)">Contraseña</label>
            <input
              type="password"
              className="w-full px-3.5 py-2.5 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)] focus:bg-(--surface-btn) transition-colors"
              value={password}
              onChange={e => setPassword(e.target.value)}
              required
            />
          </div>
          <button
            type="submit"
            disabled={loading}
            className="flex items-center justify-center gap-2 w-full py-2.5 bg-[var(--accent)] text-white text-sm font-medium rounded-sm hover:opacity-90 transition-opacity cursor-pointer disabled:opacity-50"
          >
            {loading ? <div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin" /> : <LogIn className="w-4 h-4" />}
            {loading ? 'Ingresando...' : 'Ingresar'}
          </button>
        </form>
      </div>
    </div>
  )
}
