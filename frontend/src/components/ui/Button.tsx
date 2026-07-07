import type { ButtonHTMLAttributes, ReactNode } from 'react'

type ButtonVariant = 'primary' | 'secondary' | 'danger' | 'ghost' | 'icon'
type ButtonSize = 'sm' | 'md' | 'lg'

interface ButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: ButtonVariant
  size?: ButtonSize
  icon?: ReactNode
  children?: ReactNode
}

const variantClasses: Record<ButtonVariant, string> = {
  primary:
    'bg-[var(--accent)] text-white hover:opacity-90',
  secondary:
    'border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover)',
  danger:
    'bg-[var(--danger)] text-white hover:opacity-90',
  ghost:
    'text-(--text-secondary) hover:text-(--text-primary) hover:bg-(--surface-hover)',
  icon:
    'text-(--text-secondary) hover:bg-(--surface-hover)',
}

const sizeClasses: Record<ButtonSize, string> = {
  sm: 'px-3 py-1.5 text-sm rounded-sm gap-1.5',
  md: 'px-4 py-2 text-sm rounded-sm gap-2',
  lg: 'px-5 py-2.5 text-sm rounded-sm gap-2',
}

const iconSizes: Record<ButtonSize, string> = {
  sm: 'p-1.5',
  md: 'p-2',
  lg: 'p-2.5',
}

export function Button({
  variant = 'secondary',
  size = 'md',
  icon,
  children,
  className = '',
  ...props
}: ButtonProps) {
  const isIconOnly = variant === 'icon' || (icon && !children)

  const base = `inline-flex items-center justify-center font-medium cursor-pointer transition-all duration-200 disabled:opacity-30 disabled:cursor-default no-underline ${className}`

  const cls = isIconOnly
    ? `${base} ${iconSizes[size]} ${variantClasses[variant]} rounded-sm`
    : `${base} ${sizeClasses[size]} ${variantClasses[variant]} rounded-sm`

  return (
    <button className={cls} {...props}>
      {icon && <span className="shrink-0">{icon}</span>}
      {children}
    </button>
  )
}
