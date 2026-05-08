import { useState, useRef } from 'react'
import { useClasses } from '../hooks/useGraph'
import { useVirtualizer } from '@tanstack/react-virtual'

interface ClassMetric {
    name: string
    line: number
    methods: number
    complexity: number
}

interface Props {
    onSelect: (name: string) => void
}

const ROW_HEIGHT = 36
const HEADER_HEIGHT = 36

export default function ClassesTable({ onSelect }: Props) {
    const { data, isLoading, error } = useClasses()
    const [filter, setFilter] = useState('')
    const parentRef = useRef<HTMLDivElement>(null)

    if (error) return <div style={{ color: 'var(--text-dim)', padding: 40 }}>⚠️ Error loading classes</div>
    if (isLoading) return <div style={{ color: 'var(--text-dim)', padding: 40 }}>Loading classes…</div>

    const filtered = (data ?? []).filter((c: ClassMetric) => c.name.toLowerCase().includes(filter.toLowerCase()))

    const rowVirtualizer = useVirtualizer({
        count: filtered.length,
        getScrollElement: () => parentRef.current,
        estimateSize: () => ROW_HEIGHT,
        overscan: 10,
    })

    const columns = [
        { key: 'name', label: 'Name', width: 200 },
        { key: 'line', label: 'Line', width: 80 },
        { key: 'methods', label: 'Methods', width: 100 },
        { key: 'complexity', label: 'Complexity', width: 100 },
    ]

    return (
        <div className="fade-in" style={{ display: 'flex', flexDirection: 'column', height: '100%' }}>
            <div style={{ display: 'flex', gap: 12, marginBottom: 12 }}>
                <input
                    placeholder="Search classes…"
                    value={filter}
                    onChange={e => setFilter(e.target.value)}
                    style={{ width: 260, background: '#1e1e2e', border: '1px solid #444', borderRadius: 4, padding: '4px 8px', color: '#ccc' }}
                />
            </div>
            <div style={{ display: 'flex', borderBottom: '1px solid var(--border)', height: HEADER_HEIGHT, alignItems: 'center', fontWeight: 500, color: 'var(--header)' }}>
                {columns.map(col => (
                    <div key={col.key} style={{ width: col.width, padding: '0 8px' }}>{col.label}</div>
                ))}
            </div>
            <div ref={parentRef} style={{ flex: 1, overflow: 'auto' }}>
                <div style={{ height: rowVirtualizer.getTotalSize(), position: 'relative' }}>
                    {rowVirtualizer.getVirtualItems().map(virtualRow => {
                        const c = filtered[virtualRow.index]
                        return (
                            <div
                                key={c.name}
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
                                onClick={() => onSelect(c.name)}
                            >
                                <div style={{ width: 200, padding: '0 8px', color: 'var(--accent)', fontWeight: 500 }}>{c.name}</div>
                                <div style={{ width: 80, padding: '0 8px', color: 'var(--text-secondary)', textAlign: 'right' }}>{c.line}</div>
                                <div style={{ width: 100, padding: '0 8px', textAlign: 'right' }}>{c.methods}</div>
                                <div style={{ width: 100, padding: '0 8px', textAlign: 'right' }}>{c.complexity}</div>
                            </div>
                        )
                    })}
                </div>
            </div>
        </div>
    )
}