import { useState, useRef } from 'react'
import { useFunctions } from '../hooks/useGraph'
import { useVirtualizer } from '@tanstack/react-virtual'
import type { FunctionMetric } from '../api/graph'

interface Props {
    onSelect: (name: string) => void
}

const ROW_HEIGHT = 36
const HEADER_HEIGHT = 36

export default function FunctionsTable({ onSelect }: Props) {
    const { data, isLoading, error } = useFunctions()
    const [filter, setFilter] = useState('')
    const [sortKey, setSortKey] = useState<keyof FunctionMetric>('name')
    const [sortAsc, setSortAsc] = useState(true)
    const parentRef = useRef<HTMLDivElement>(null)

    if (error) return <div style={{ color: 'var(--text-dim)', padding: 40 }}>⚠️ Error loading functions</div>
    if (isLoading) return <div style={{ color: 'var(--text-dim)', padding: 40 }}>Loading functions…</div>

    const filtered = (data ?? []).filter(f => f.name.toLowerCase().includes(filter.toLowerCase()))
    const sorted = [...filtered].sort((a, b) => {
        const va = a[sortKey]
        const vb = b[sortKey]
        let cmp = 0
        if (typeof va === 'number' && typeof vb === 'number') cmp = va - vb
        else if (typeof va === 'boolean' && typeof vb === 'boolean') cmp = Number(va) - Number(vb)
        else cmp = String(va).localeCompare(String(vb))
        return sortAsc ? cmp : -cmp
    })

    const toggleSort = (key: keyof FunctionMetric) => {
        if (sortKey === key) setSortAsc(!sortAsc)
        else { setSortKey(key); setSortAsc(true) }
    }

    const rowVirtualizer = useVirtualizer({
        count: sorted.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => ROW_HEIGHT,
        overscan: 10,
    })

    const columns = [
        { key: 'name', label: 'Name', width: 160 },
        { key: 'kind', label: 'Kind', width: 80 },
        { key: 'line', label: 'Line', width: 60 },
        { key: 'complexity', label: 'Complexity', width: 100 },
        { key: 'loc', label: 'LOC', width: 70 },
        { key: 'callers', label: 'Callers', width: 70 },
        { key: 'callees', label: 'Callees', width: 70 },
        { key: 'status', label: 'Status', width: 120 },
    ]

    return (
        <div className="fade-in" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            <div style={{ display: 'flex', gap: 12, marginBottom: 12 }}>
                <input
                    placeholder="Search functions…"
                    value={filter}
                    onChange={e => setFilter(e.target.value)}
                    style={{ width: 260, background: '#1e1e2e', border: '1px solid #444', borderRadius: 4, padding: '4px 8px', color: '#ccc' }}
                />
            </div>
            <div style={{ display: 'flex', borderBottom: '1px solid var(--border)', height: HEADER_HEIGHT, alignItems: 'center', fontWeight: 500, color: 'var(--header)' }}>
                {columns.map(col => {
                    if (col.key === 'status') {
                        return <div key={col.key} style={{ width: col.width, padding: '0 8px' }}>{col.label}</div>
                    }
                    const isActive = sortKey === col.key
                    return (
                        <div
                            key={col.key}
                            style={{ width: col.width, padding: '0 8px', cursor: 'pointer', userSelect: 'none' }}
                            onClick={() => toggleSort(col.key as keyof FunctionMetric)}
                        >
                            {col.label} {isActive ? (sortAsc ? '↑' : '↓') : ''}
                        </div>
                    )
                })}
            </div>
            <div ref={parentRef} style={{ flex: 1, overflow: 'auto' }}>
                <div style={{ height: rowVirtualizer.getTotalSize(), position: 'relative' }}>
                    {rowVirtualizer.getVirtualItems().map(virtualRow => {
                        const f = sorted[virtualRow.index]
                        return (
                            <div
                                key={f.name}
                                style={{
                                    position: 'absolute',
                                    top: 0,
                                    left: 0,
                                    width: '100%',
                                    height: ROW_HEIGHT,
                                    transform: `translateY(${virtualRow.start}px)`,
                                    display: 'flex',
                                    alignItems: 'center',
                                    borderBottom: '1px solid var(--border)',
                                    boxSizing: 'border-box',
                                    cursor: 'pointer',
                                }}
                                onClick={() => onSelect(f.name)}
                            >
                                <div style={{ width: 160, padding: '0 8px', color: 'var(--accent)', fontWeight: 500, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{f.name}</div>
                                <div style={{ width: 80, padding: '0 8px', color: 'var(--text-secondary)' }}>{f.kind}</div>
                                <div style={{ width: 60, padding: '0 8px', color: 'var(--text-secondary)', textAlign: 'right' }}>{f.line}</div>
                                <div style={{ width: 100, padding: '0 8px', textAlign: 'right' }}>{f.complexity}</div>
                                <div style={{ width: 70, padding: '0 8px', textAlign: 'right' }}>{f.loc}</div>
                                <div style={{ width: 70, padding: '0 8px', textAlign: 'right' }}>{f.callers}</div>
                                <div style={{ width: 70, padding: '0 8px', textAlign: 'right' }}>{f.callees}</div>
                                <div style={{ width: 120, padding: '0 8px', display: 'flex', gap: 4 }}>
                                    {f.is_cycle && <StatusBadge label="cycle" color="#ff6b6b" />}
                                    {f.is_dead && <StatusBadge label="dead" color="#94a3b8" />}
                                </div>
                            </div>
                        )
                    })}
                </div>
            </div>
        </div>
    )
}

function StatusBadge({ label, color }: { label: string; color: string }) {
    return (
        <span style={{
            display: 'inline-block',
            padding: '1px 6px',
            borderRadius: 10,
            fontSize: 11,
            fontWeight: 500,
            background: `${color}20`,
            color: color,
        }}>
            {label}
        </span>
    )
}