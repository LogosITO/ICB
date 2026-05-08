import { useState, useRef, useEffect, useCallback } from 'react'
import ELK from 'elkjs/lib/elk.bundled.js'

interface DiffNode {
    name: string
    kind: string
    line: number
    status: 'Added' | 'Removed' | 'Unchanged'
}

interface DiffEdge {
    source: string
    target: string
    status: 'Added' | 'Removed'
}

interface ElkNode {
    id: string
    width: number
    height: number
    data: DiffNode
    x?: number
    y?: number
}

interface ElkEdge {
    id: string
    sources: string[]
    targets: string[]
}

const elk = new ELK()
const STATUS_COLORS: Record<string, string> = {
    Added: '#4caf50',
    Removed: '#f44336',
    Unchanged: '#9e9e9e',
}
const EDGE_COLORS: Record<string, string> = {
    Added: 'rgba(76,175,80,0.4)',
    Removed: 'rgba(244,67,54,0.4)',
}
const EDGE_MARKER_COLORS: Record<string, string> = {
    Added: '#4caf50',
    Removed: '#f44336',
}

export default function DiffViewer() {
    const [oldPath, setOldPath] = useState('')
    const [newPath, setNewPath] = useState('')
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState('')
    const [layout, setLayout] = useState<{ children?: ElkNode[]; edges?: ElkEdge[] } | null>(null)
    const svgRef = useRef<SVGSVGElement>(null)
    const [scale, setScale] = useState(1)
    const [offset, setOffset] = useState({ x: 0, y: 0 })
    const isDragging = useRef(false)
    const lastPos = useRef({ x: 0, y: 0 })

    const fetchDiff = useCallback(async () => {
        setLoading(true)
        setError('')
        try {
            const res = await fetch(`/api/diff?old=${encodeURIComponent(oldPath)}&new=${encodeURIComponent(newPath)}`)
            if (!res.ok) throw new Error(await res.text())
            const json = await res.json()
            buildLayout(json)
        } catch (e: any) {
            setError(e.message)
        } finally {
            setLoading(false)
        }
    }, [oldPath, newPath])

    const buildLayout = async (data: { nodes: DiffNode[]; edges: DiffEdge[] }) => {
        if (!data.nodes.length) {
            setLayout(null)
            return
        }
        const nodeMap = new Map<string, number>()
        const elkNodes: ElkNode[] = data.nodes.map((n, idx) => {
            nodeMap.set(n.name, idx)
            return {
                id: idx.toString(),
                width: 200,
                height: 48,
                data: n,
            }
        })
        const elkEdges: ElkEdge[] = data.edges
            .map((e, idx) => {
                const src = nodeMap.get(e.source)
                const tgt = nodeMap.get(e.target)
                if (src === undefined || tgt === undefined) return null
                return {
                    id: `${src}->${tgt}-${idx}`,
                    sources: [src.toString()],
                    targets: [tgt.toString()],
                }
            })
            .filter(Boolean) as ElkEdge[]

        const graph = {
            id: 'root',
            layoutOptions: {
                'elk.algorithm': 'layered',
                'elk.direction': 'RIGHT',
                'elk.spacing.nodeNode': '80',
                'elk.layered.spacing.nodeNodeBetweenLayers': '120',
            },
            children: elkNodes,
            edges: elkEdges,
        }
        const result = await elk.layout(graph)
        setLayout({ children: result.children as ElkNode[], edges: result.edges as ElkEdge[] })
    }

    useEffect(() => {
        if (oldPath && newPath) fetchDiff()
    }, [])

    const handleWheel = (e: React.WheelEvent) => {
        e.preventDefault()
        setScale(s => Math.max(0.2, Math.min(2, s * (e.deltaY > 0 ? 0.9 : 1.1))))
    }
    const handleMouseDown = (e: React.MouseEvent) => {
        isDragging.current = true
        lastPos.current = { x: e.clientX, y: e.clientY }
    }
    const handleMouseMove = (e: React.MouseEvent) => {
        if (!isDragging.current) return
        setOffset(o => ({
            x: o.x + e.clientX - lastPos.current.x,
            y: o.y + e.clientY - lastPos.current.y,
        }))
        lastPos.current = { x: e.clientX, y: e.clientY }
    }
    const handleMouseUp = () => { isDragging.current = false }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: 'calc(100vh - 80px)', background: '#111' }}>
            <div style={{ display: 'flex', gap: '1rem', padding: '1rem', background: '#1a1a1a', borderBottom: '1px solid #333' }}>
                <input
                    placeholder="Old project path"
                    value={oldPath}
                    onChange={e => setOldPath(e.target.value)}
                    style={{ flex: 1, padding: '0.5rem', background: '#111', border: '1px solid #333', borderRadius: 4, color: '#ccc' }}
                />
                <input
                    placeholder="New project path"
                    value={newPath}
                    onChange={e => setNewPath(e.target.value)}
                    style={{ flex: 1, padding: '0.5rem', background: '#111', border: '1px solid #333', borderRadius: 4, color: '#ccc' }}
                />
                <button onClick={fetchDiff} style={{ padding: '0.5rem 1.5rem', background: '#b0b0b0', border: 'none', borderRadius: 4, color: '#111', fontWeight: 600, cursor: 'pointer' }}>
                    Compare
                </button>
            </div>
            {loading && <div style={{ color: '#888', padding: '1rem' }}>Loading…</div>}
            {error && <div style={{ color: '#f44', padding: '1rem' }}>{error}</div>}
            <div style={{ flex: 1, position: 'relative', overflow: 'hidden' }}>
                <div style={{ position: 'absolute', top: 8, left: 8, zIndex: 10, display: 'flex', gap: 12, background: 'rgba(18,24,32,0.9)', padding: '4px 12px', borderRadius: 4, border: '1px solid #444' }}>
                    {Object.entries(STATUS_COLORS).map(([label, color]) => (
                        <div key={label} style={{ display: 'flex', alignItems: 'center', gap: 4 }}>
                            <div style={{ width: 12, height: 12, background: color, borderRadius: 2 }} />
                            <span style={{ fontSize: '0.75rem', color: '#888' }}>{label}</span>
                        </div>
                    ))}
                </div>
                {layout && (
                    <div
                        style={{ width: '100%', height: '100%', cursor: 'grab' }}
                        onWheel={handleWheel}
                        onMouseDown={handleMouseDown}
                        onMouseMove={handleMouseMove}
                        onMouseUp={handleMouseUp}
                    >
                        <svg ref={svgRef} width="100%" height="100%">
                            <defs>
                                {Object.entries(EDGE_MARKER_COLORS).map(([status, color]) => (
                                    <marker key={status} id={`arrow-${status}`} viewBox="0 0 10 10" refX="9" refY="5" markerWidth="6" markerHeight="6" orient="auto">
                                        <path d="M 0 0 L 10 5 L 0 10 z" fill={color} opacity="0.8" />
                                    </marker>
                                ))}
                            </defs>
                            <g transform={`translate(${offset.x},${offset.y}) scale(${scale})`}>
                                {layout.edges?.map(e => {
                                    const srcNode = layout.children?.find(n => n.id === e.sources[0])
                                    const tgtNode = layout.children?.find(n => n.id === e.targets[0])
                                    if (!srcNode || !tgtNode || srcNode.x === undefined || tgtNode.x === undefined) return null
                                    const sx = srcNode.x + srcNode.width
                                    const sy = srcNode.y! + srcNode.height / 2
                                    const tx = tgtNode.x
                                    const ty = tgtNode.y! + tgtNode.height / 2
                                    const dx = tx - sx
                                    const cx1 = sx + dx * 0.4
                                    const cx2 = tx - dx * 0.4
                                    const edgeStatus = 'Added' // у нас нет статуса ребра в данных, можно захардкодить или добавить
                                    return (
                                        <path
                                            key={e.id}
                                            d={`M ${sx},${sy} C ${cx1},${sy} ${cx2},${ty} ${tx},${ty}`}
                                            stroke={EDGE_COLORS[edgeStatus]}
                                            strokeWidth={2}
                                            fill="none"
                                            markerEnd={`url(#arrow-${edgeStatus})`}
                                        />
                                    )
                                })}
                                {layout.children?.map(n => {
                                    const color = STATUS_COLORS[n.data.status] || '#9e9e9e'
                                    return (
                                        <g key={n.id} transform={`translate(${n.x ?? 0}, ${n.y ?? 0})`}>
                                            <rect width={n.width} height={n.height} rx={4} fill="#121820" stroke={color} strokeWidth={2} />
                                            <text x={n.width / 2} y={20} fill="#e6edf3" fontSize="12" textAnchor="middle" fontFamily="monospace">
                                                {n.data.name}
                                            </text>
                                            <text x={n.width / 2} y={36} fill="#8b949e" fontSize="10" textAnchor="middle">
                                                {n.data.kind}:{n.data.line}
                                            </text>
                                        </g>
                                    )
                                })}
                            </g>
                        </svg>
                    </div>
                )}
            </div>
        </div>
    )
}