import { apiGet } from './client'

export interface AppConfig {
  polling_interval_ms: number
}

export function getConfig(): Promise<AppConfig> {
  return apiGet<AppConfig>('/config')
}
