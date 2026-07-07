interface TopbarProps {
  title: string
}

export function Topbar({ title }: TopbarProps) {
  return (
    <header className="h-[var(--header-height)] px-3 md:px-8 flex items-center justify-between sticky top-0 z-30 bg-[var(--topbar-bg)] backdrop-blur-[12px] border-b border-(--glass-border)">
      <div className="flex items-center gap-3 min-w-0">
        <h1 className="text-sm md:text-lg font-semibold text-(--text-primary) truncate">{title}</h1>
      </div>
    </header>
  )
}
