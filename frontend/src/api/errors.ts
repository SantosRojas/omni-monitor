export class ApiError extends Error {
  status: number

  constructor(status: number, message: string) {
    super(message)
    this.name = 'ApiError'
    this.status = status
  }
}

export async function parseApiError(res: Response): Promise<ApiError> {
  let message: string
  try {
    const body = await res.json()
    message = body?.error ?? body?.message ?? res.statusText
  } catch {
    const text = await res.text().catch(() => res.statusText)
    message = text || res.statusText
  }
  return new ApiError(res.status, message)
}
