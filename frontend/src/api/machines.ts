import type { Machine, MachineIp, MachineIpWithSerial, CreateMachineIpRequest, UpdateMachineIpRequest } from '../types'
import { apiDelete, apiGet, apiPost, apiPut } from './client'

export function listMachines(): Promise<Machine[]> {
  return apiGet<Machine[]>('/machines')
}

export function listMachineIps(): Promise<MachineIpWithSerial[]> {
  return apiGet<MachineIpWithSerial[]>('/admin/machine-ips')
}

export function createMachineIp(req: CreateMachineIpRequest): Promise<MachineIp> {
  return apiPost<MachineIp>('/admin/machine-ips', req)
}

export function updateMachineIp(id: number, req: UpdateMachineIpRequest): Promise<MachineIp> {
  return apiPut<MachineIp>(`/admin/machine-ips/${id}`, req)
}

export function deleteMachineIp(id: number): Promise<void> {
  return apiDelete<void>(`/admin/machine-ips/${id}`)
}
