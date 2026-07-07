import { forwardRef, useState, type InputHTMLAttributes, type ReactNode } from 'react'
import { Eye, EyeOff } from 'lucide-react'

type InputVariant = 'form' | 'search' | 'login' | 'column-filter' | 'tiny'

interface InputProps extends InputHTMLAttributes<HTMLInputElement> {
  variant?: InputVariant
  leftIcon?: ReactNode
  showPasswordToggle?: boolean
}

const variantClasses: Record<InputVariant, string> = {
  form:
    'w-full px-3 py-2 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)]',
  search:
    'w-full pl-9 pr-3 py-2 text-sm border border-(--glass-border) rounded-sm bg-(--surface-btn) text-(--text-primary) outline-none focus:border-[var(--accent)]',
  login:
    'w-full px-3.5 py-2.5 bg-(--surface-btn) border border-(--glass-border) rounded-sm text-sm text-(--text-primary) outline-none focus:border-[var(--accent)] focus:bg-(--surface-btn) transition-colors',
  'column-filter':
    'w-full px-2 py-1 text-xs border border-(--glass-border) rounded-sm bg-(--surface-btn) text-(--text-primary) outline-none focus:border-[var(--accent)] mt-1.5',
  tiny:
    'flex-1 min-w-0 px-1.5 py-1 text-xs rounded-sm bg-(--surface-btn) border border-(--border-subtle) text-(--text-primary) outline-none',
}

export const Input = forwardRef<HTMLInputElement, InputProps>(
  ({ variant = 'form', leftIcon, showPasswordToggle = true, className = '', ...props }, ref) => {
    const [visible, setVisible] = useState(false)
    const isPassword = props.type === 'password'
    const needsWrapper = !!(leftIcon || (isPassword && showPasswordToggle))

    const input = (
      <input
        ref={ref}
        {...props}
        type={isPassword ? (visible ? 'text' : 'password') : props.type}
        className={`${variantClasses[variant]} ${isPassword && showPasswordToggle ? 'pr-9' : ''} ${className}`}
      />
    )

    if (!needsWrapper) return input

    return (
      <div className="relative">
        {leftIcon && (
          <div className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-(--text-muted) pointer-events-none">
            {leftIcon}
          </div>
        )}
        {input}
        {isPassword && showPasswordToggle && (
          <button
            type="button"
            tabIndex={-1}
            onClick={() => setVisible(v => !v)}
            className="absolute right-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-(--text-muted) hover:text-(--text-secondary) cursor-pointer"
          >
            {visible ? <EyeOff className="w-4 h-4" /> : <Eye className="w-4 h-4" />}
          </button>
        )}
      </div>
    )
  },
)

Input.displayName = 'Input'
