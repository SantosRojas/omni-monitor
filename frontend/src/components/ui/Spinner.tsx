export function Spinner({ message }: { message?: string }) {
  return (
    <div className="flex flex-col items-center justify-center py-10 text-(--text-muted)">
      <div className="w-9 h-9 border-3 border-white/10 border-t-[var(--accent)] rounded-full animate-spin mb-3" />
      {message && <p className="text-sm">{message}</p>}
    </div>
  )
}
