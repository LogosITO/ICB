import { useNodeDetail } from '../hooks/useGraph'
import { X } from 'lucide-react'

export default function NodeDetailPanel({ name, onClose }: { name: string; onClose: () => void }) {
    const { data, isLoading } = useNodeDetail(name)

    if (isLoading) return <div className="text-gray-400 p-4">Loading...</div>
    if (!data) return <div className="text-red-400 p-4">Not found</div>

    return (
        <div className="flex flex-col gap-4 text-sm">
            <div className="flex justify-between items-center">
                <h3 className="text-lg font-bold text-blue-400">{data.name}</h3>
                <button onClick={onClose} className="text-gray-400 hover:text-white">
                    <X size={18} />
                </button>
            </div>
            <div className="grid grid-cols-2 gap-2">
                <span className="text-gray-500">Kind</span>
                <span className="bg-blue-900 text-blue-300 px-2 py-0.5 rounded-full text-xs text-center">{data.kind}</span>
                <span className="text-gray-500">Line</span>
                <span>{data.line}</span>
                <span className="text-gray-500">File</span>
                <span className="truncate">{data.file || 'N/A'}</span>
            </div>
            <div>
                <h4 className="font-semibold text-gray-300">Callers</h4>
                <p className="text-gray-400">{data.callers?.join(', ') || 'none'}</p>
            </div>
            <div>
                <h4 className="font-semibold text-gray-300">Callees</h4>
                <p className="text-gray-400">{data.callees?.join(', ') || 'none'}</p>
            </div>
            <div className="flex gap-4">
                {data.is_cycle && <span className="bg-red-900 text-red-300 px-2 py-0.5 rounded-full text-xs">⚠️ Cycle</span>}
                {data.is_dead && <span className="bg-gray-700 text-gray-300 px-2 py-0.5 rounded-full text-xs">💀 Dead</span>}
            </div>
        </div>
    )
}