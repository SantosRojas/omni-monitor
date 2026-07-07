import { ChevronLeft, ChevronRight } from 'lucide-react'
import type { Table } from '@tanstack/react-table'

interface PaginationProps<T> {
  table: Table<T>
}

export function Pagination<T>({ table }: PaginationProps<T>) {
  return (
    <div className="flex items-center justify-center gap-1 sm:gap-2 pt-4 pb-2">
      <button
        onClick={() => table.previousPage()}
        disabled={!table.getCanPreviousPage()}
        className="flex items-center gap-1 px-2 sm:px-3 py-1.5 text-xs sm:text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) disabled:opacity-30 cursor-pointer disabled:cursor-default transition-colors"
      >
        <ChevronLeft className="w-3.5 h-3.5" /> <span className="hidden sm:inline">Anterior</span>
      </button>
      <span className="text-xs sm:text-sm text-(--text-muted)">
        {table.getState().pagination.pageIndex + 1} / {table.getPageCount()}
      </span>
      <button
        onClick={() => table.nextPage()}
        disabled={!table.getCanNextPage()}
        className="flex items-center gap-1 px-2 sm:px-3 py-1.5 text-xs sm:text-sm rounded-sm border border-(--glass-border) bg-(--surface-btn) text-(--text-secondary) hover:bg-(--surface-btn-hover) disabled:opacity-30 cursor-pointer disabled:cursor-default transition-colors"
      >
        <span className="hidden sm:inline">Siguiente</span> <ChevronRight className="w-3.5 h-3.5" />
      </button>
    </div>
  )
}
