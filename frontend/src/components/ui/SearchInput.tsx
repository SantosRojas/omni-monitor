import { Search } from 'lucide-react'
import { Input } from './Input'
import type { InputHTMLAttributes } from 'react'

interface SearchInputProps extends InputHTMLAttributes<HTMLInputElement> {
  value: string
  onChange: (e: React.ChangeEvent<HTMLInputElement>) => void
  placeholder?: string
}

export function SearchInput({ value, onChange, placeholder = 'Buscar...', className = '', ...props }: SearchInputProps) {
  return (
    <Input
      variant="search"
      leftIcon={<Search className="w-4 h-4" />}
      value={value}
      onChange={onChange}
      placeholder={placeholder}
      className={className}
      {...props}
    />
  )
}
