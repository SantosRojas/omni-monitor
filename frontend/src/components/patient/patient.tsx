import { User } from "lucide-react"

interface PatientProps {
    id: number
    patient_id: string
    onClick?: () => void
}


export const PatientComponent = ({ patient_id, onClick }: PatientProps) => {

    return (
        <button type="button" onClick={onClick} className="flex flex-col gap-1 items-center justify-center rounded-md p-1 hover:bg-(--surface-row-hover) hover:shadow-sm transition-all cursor-pointer">
            <User className="w-10 h-10 text-(--accent)" />
            <p className="text-sm text-(--text-secondary) font-semibold font-mono">{patient_id}</p>
        </button>
    )
}