import { useState } from 'react'
import { useQuery } from '@tanstack/react-query'
import { fetchGraph } from '../api/graph'
import SearchInput from './SearchInput'
import SettingsPanel from './SettingsPanel'
import { GraphParams } from '../App'

export default function Sidebar({
                                    params,
                                    onApply,
                                    onSelectNode,
                                }: {
    params: GraphParams
    onApply: (p: GraphParams) => void
    onSelectNode: (name: string) => void
}) {
    const [search, setSearch] = useState('')
    const { data } = useQuery({
        queryKey: ['functions'],
        queryFn: () => fetchGraph({ kind: 'Function', max_nodes: '2000' }),
        staleTime: 300_000,
    })
    const functions = data?.nodes?.filter(n => n.name) ?? []
    const filtered = search ? functions.filter(f => f.name?.toLowerCase().includes(search.toLowerCase())) : functions

    return (
        <>
            <h2 className="text-xl font-bold text-blue-400">🔍 ICB Navigator</h2>
            <SearchInput value={search} onChange={setSearch} />
            <SettingsPanel params={params} onChange={onApply} />
            <button
                onClick={() => onApply({ ...params, focus: undefined, kind: 'Function' })}
                className="bg-green-600 hover:bg-green-500 text-white font-semibold py-2 px-4 rounded"
            >
                🌐 Full Overview
            </button>
            <div className="flex-1 overflow-y-auto border border-gray-700 rounded p-1 bg-gray-950">
                {filtered.slice(0, 500).map((f, idx) => (
                    <div
                        key={`${f.name}-${idx}`}
                        className={`p-1.5 text-sm cursor-pointer hover:bg-gray-800 rounded ${
                            params.focus === f.name ? 'bg-blue-900' : ''
                        }`}
                        onClick={() => {
                            onApply({ ...params, focus: f.name! })
                            onSelectNode(f.name!)
                        }}
                    >
                        {f.name} <span className="text-gray-500">(line {f.start_line})</span>
                    </div>
                ))}
            </div>
        </>
    )
}