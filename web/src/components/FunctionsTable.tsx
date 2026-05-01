import { useState } from 'react'
import { useFunctions } from '../hooks/useGraph'
import type { FunctionMetric } from '../api/graph'

interface Props {
    onSelect: (name: string) => void
}

export default function FunctionsTable({ onSelect }: Props) {
    const { data, isLoading } = useFunctions()
    const [filter, setFilter] = useState('')
    const [sortKey, setSortKey] = useState<keyof FunctionMetric>('name')
    const [sortAsc, setSortAsc] = useState(true)

    if (isLoading) return <div style={{ color: 'var(--text-dim)' }}>Loading…</div>

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

    return (
        <div>
            <div style={{ display: 'flex', gap: '12px', marginBottom: '16px' }}>
                <input
                    placeholder="Filter functions…"
                    value={filter}
                    onChange={e => setFilter(e.target.value)}
                    style={{ width: '240px' }}
                />
            </div>
            <table>
                <thead>
                <tr>
                    <SortableHeader label="Name" sortKey="name" currentKey={sortKey} asc={sortAsc} onClick={toggleSort} />
                    <SortableHeader label="Kind" sortKey="kind" currentKey={sortKey} asc={sortAsc} onClick={toggleSort} />
                    <SortableHeader label="Line" sortKey="line" currentKey={sortKey} asc={sortAsc} onClick={toggleSort} />
                    <SortableHeader label="Complexity" sortKey="complexity" currentKey={sortKey} asc={sortAsc} onClick={toggleSort} />
                    <SortableHeader label="Callers" sortKey="callers" currentKey={sortKey} asc={sortAsc} onClick={toggleSort} />
                    <SortableHeader label="Callees" sortKey="callees" currentKey={sortKey} asc={sortAsc} onClick={toggleSort} />
                    <th>Status</th>
                </tr>
                </thead>
                <tbody>
                {sorted.map(f => (
                    <tr key={f.name} onClick={() => onSelect(f.name)} style={{ cursor: 'pointer' }}>
                        <td style={{ color: 'var(--accent)', fontWeight: 500 }}>{f.name}</td>
                        <td style={{ color: 'var(--text-dim)' }}>{f.kind}</td>
                        <td style={{ color: 'var(--text-dim)' }}>{f.line}</td>
                        <td>{f.complexity}</td>
                        <td>{f.callers}</td>
                        <td>{f.callees}</td>
                        <td>
                <span style={{ display: 'flex', gap: '4px' }}>
                  {f.is_cycle && <StatusBadge label="cycle" color="#ff6b6b" />}
                    {f.is_dead && <StatusBadge label="dead" color="#94a3b8" />}
                </span>
                        </td>
                    </tr>
                ))}
                </tbody>
            </table>
        </div>
    )
}

function SortableHeader({ label, sortKey, currentKey, asc, onClick }: {
    label: string
    sortKey: keyof FunctionMetric
    currentKey: string
    asc: boolean
    onClick: (key: keyof FunctionMetric) => void
}) {
    const isActive = sortKey === currentKey
    return (
        <th onClick={() => onClick(sortKey)} style={{ cursor: 'pointer', userSelect: 'none' }}>
            {label} {isActive ? (asc ? '↑' : '↓') : ''}
        </th>
    )
}

function StatusBadge({ label, color }: { label: string; color: string }) {
    return (
        <span style={{
            display: 'inline-block',
            padding: '1px 6px',
            borderRadius: '10px',
            fontSize: '11px',
            fontWeight: 500,
            background: `${color}20`,
            color: color,
        }}>
      {label}
    </span>
    )
}