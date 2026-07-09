import { useState } from 'react'
import { useNavigate } from 'react-router-dom'
import { Activity, LogIn } from 'lucide-react'
import { useAuth } from '../contexts/AuthContext'
import * as authApi from '../api/auth'
import { Input, Label, Button } from '../components/ui'

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
      login(res.user)
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
          <Activity className="w-10 h-10 text-(--accent) mb-2" />
          <h1 className="text-2xl font-bold text-(--text-primary)">Monitor OMNI</h1>
          <p className="text-sm text-(--text-muted)">Inicia sesión para continuar</p>
        </div>

        {error && (
          <div className="bg-red-500/15 border border-red-500/30 text-(--danger) text-sm text-center px-4 py-2.5 rounded-sm mb-4">
            {error}
          </div>
        )}

        <form onSubmit={handleSubmit} className="flex flex-col gap-4">
          <div>
            <Label className="mb-1.5">Usuario</Label>
            <Input variant="login" autoComplete="username" value={username} onChange={e => setUsername(e.target.value)} autoFocus required />
          </div>
          <div>
            <Label className="mb-1.5">Contraseña</Label>
            <Input variant="login" type="password" autoComplete="current-password" value={password} onChange={e => setPassword(e.target.value)} required />
          </div>
          <Button type="submit" variant="primary" size="lg" className="w-full!" disabled={loading} icon={loading ? undefined : <LogIn className="w-4 h-4" />}>
            {loading ? (
              <><div className="w-4 h-4 border-2 border-white/30 border-t-white rounded-full animate-spin shrink-0" /> Ingresando...</>
            ) : (
              'Ingresar'
            )}
          </Button>
        </form>
      </div>
    </div>
  )
}
