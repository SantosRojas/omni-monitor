import { createContext, useContext, useEffect, useState, type ReactNode } from 'react'
import type { UserResponse } from '../types'
import * as authApi from '../api/auth'

interface AuthContextType {
  token: string | null
  user: UserResponse | null
  isLoggedIn: boolean
  isAdmin: boolean
  login: (token: string, user: UserResponse) => void
  logout: () => void
}

const AuthContext = createContext<AuthContextType | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setToken] = useState<string | null>(() => localStorage.getItem('monitor_token'))
  const [user, setUser] = useState<UserResponse | null>(() => {
    const raw = localStorage.getItem('monitor_user')
    if (!raw) return null
    try { return JSON.parse(raw) }
    catch { return null }
  })

  useEffect(() => {
    if (token && !user) {
      authApi.getMe(token).then(setUser).catch(() => logout())
    }
  }, [])

  const login = (t: string, u: UserResponse) => {
    setToken(t)
    setUser(u)
    localStorage.setItem('monitor_token', t)
    localStorage.setItem('monitor_user', JSON.stringify(u))
  }

  const logout = () => {
    setToken(null)
    setUser(null)
    localStorage.removeItem('monitor_token')
    localStorage.removeItem('monitor_user')
  }

  return (
    <AuthContext.Provider value={{
      token, user,
      isLoggedIn: !!token,
      isAdmin: user?.role === 'admin',
      login, logout,
    }}>
      {children}
    </AuthContext.Provider>
  )
}

export function useAuth() {
  const ctx = useContext(AuthContext)
  if (!ctx) throw new Error('useAuth must be used within AuthProvider')
  return ctx
}
