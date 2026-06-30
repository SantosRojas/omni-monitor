interface BadgeProps {
  variant?: 'active' | 'inactive' | 'completed' | 'admin' | 'operator' | 'viewer' | 'default'
  children: React.ReactNode
}

const styles: Record<string, string> = {
  active: 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30',
  inactive: 'bg-red-500/20 text-red-400 border border-red-500/30',
  completed: 'bg-sky-500/20 text-sky-400 border border-sky-500/30',
  admin: 'bg-[var(--accent)]/20 text-[var(--accent)] border border-[var(--accent)]/30',
  operator: 'bg-amber-500/20 text-amber-400 border border-amber-500/30',
  viewer: 'bg-(--surface-btn) text-(--text-secondary) border border-[var(--border-subtle)]',
  default: 'bg-(--surface-btn) text-(--text-secondary) border border-[var(--border-subtle)]',
}

export function Badge({ variant = 'default', children }: BadgeProps) {
  return (
    <span className={`inline-block px-3 py-0.5 rounded-full text-xs font-semibold uppercase tracking-wide ${styles[variant]}`}>
      {children}
    </span>
  )
}
