export interface UserResponse {
  id: number
  username: string
  full_name: string
  email: string
  role: string
  active: boolean
}

export interface Patient {
  id: number
  patient_id_str: string
  created_at?: string
  therapy_start?: string
  therapy_end?: string
  active_therapy_count?: number
  completed_therapy_count?: number
}

export interface Machine {
  id: number
  serial_number: string
  software_version: string
  registered_at?: string
  status?: string
}

export interface TherapyWithMachine {
  id: number
  started_at?: string
  ended_at?: string
  status?: string
  machine_id?: number
  serial_number?: string
  software_version?: string
  ip_address?: string
  port?: number
  therapy_type?: string
  kit?: string
  weight_initial?: string
  weight_final?: string
  patient_id_str?: string
}

export interface MachineIp {
  id: number
  machine_id: number
  ip_address: string
  port?: number
  label?: string
  is_active: boolean
  created_at?: string
  updated_at?: string
}

export interface MachineIpWithSerial {
  id: number
  machine_id: number
  ip_address: string
  port?: number
  label?: string
  is_active: boolean
  created_at?: string
  updated_at?: string
  serial_number?: string
}

export interface TelemetryReading {
  id: number
  timestamp?: string
  therapy_id?: number
  signal_id?: number
  raw_value?: number
  physical_value?: string
  unit?: string
  signal_name?: string
}

export interface Signal {
  id: number
  internal_name: string
  display_name?: string
  unit?: string
}

export interface Equivalence {
  signal_id: number
  internal_name: string
  numeric_value: number
  display_name: string
}

export interface CreateEquivalenceRequest {
  internal_name: string
  numeric_value: number
  display_name: string
}

export interface UpdateEquivalenceRequest {
  signal_id: number
  numeric_value: number
  display_name: string
}

export interface UpdateSignalRequest {
  display_name?: string
  unit?: string
}

export interface TherapyComment {
  id: number
  therapy_id: number
  author_name: string
  comment: string
  created_at?: string
  deleted_at?: string
  deletion_reason?: string
}

export interface LoginResponse {
  user: UserResponse
}

export interface CreateMachineIpRequest {
  machine_id: number
  serial_number?: string
  ip_address: string
  port?: number
  label?: string
  is_active?: boolean
}

export interface UpdateMachineIpRequest {
  ip_address?: string
  port?: number
  label?: string
  is_active?: boolean
}

export interface CreateUserRequest {
  username: string
  password: string
  full_name: string
  email: string
  role: string
}

export interface UpdateUserRequest {
  password?: string
  full_name?: string
  email?: string
  role?: string
  active?: boolean
}

export interface PaginatedResponse<T> {
  data: T[]
  total: number
  page: number
  per_page: number
  total_pages: number
}

export interface DashboardSignal {
  signal_id: number
  internal_name: string
  display_name?: string
  unit?: string
  average?: number
  minimum?: number
  maximum?: number
  count: number
  values: DashboardValue[]
}

export interface DashboardValue {
  timestamp: string
  value: number
}

export interface PatientDashboard {
  signals: DashboardSignal[]
}

export interface ActiveTherapy {
  therapy_id: number
  patient_id: number
  patient_id_str: string
  started_at?: string
  serial_number?: string
  ip_address?: string
  port?: number
  arterial_pressure?: string
  venous_pressure?: string
  blood_flow?: string
  filter_pressure?: string
  tmp_pressure?: string
  effluent_pressure?: string
  net_rem_flow?: string
  fs_mid_flow?: string
  comments: TherapyComment[]
}
