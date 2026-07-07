import type { LabelHTMLAttributes, ReactNode } from 'react'

interface LabelProps extends LabelHTMLAttributes<HTMLLabelElement> {
  children: ReactNode
}

export function Label({ children, className = '', ...props }: LabelProps) {
  return (
    <label className={`block mb-1 text-xs font-medium text-(--text-secondary) ${className}`} {...props}>
      {children}
    </label>
  )
}
