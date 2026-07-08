import { createContext, useContext, useEffect, useState, useCallback, type ReactNode } from 'react'
import type { UserResponse } from '../types'
import * as authApi from '../api/auth'

interface AuthContextType {
  user: UserResponse | null
  isLoggedIn: boolean
  isAdmin: boolean
  login: (user: UserResponse) => void
  logout: () => void
}

const AuthContext = createContext<AuthContextType | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [user, setUser] = useState<UserResponse | null>(null)
  const [loading, setLoading] = useState(true)

  useEffect(() => {
    authApi.getMe()
      .then(setUser)
      .catch(() => setUser(null))
      .finally(() => setLoading(false))
  }, [])

  const login = useCallback((u: UserResponse) => {
    setUser(u)
  }, [])

  const logout = useCallback(() => {
    authApi.logout().catch(() => {})
    setUser(null)
  }, [])

  if (loading) return null

  return (
    <AuthContext.Provider value={{
      user,
      isLoggedIn: !!user,
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
