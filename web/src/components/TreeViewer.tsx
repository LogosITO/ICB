import React, { useEffect, useRef, useState, useCallback } from 'react'
import ELK from 'elkjs/lib/elk.bundled.js'

interface TreeNode {
    name: string
    kind: string
    line: number
    file?: string
    children: TreeNode[]
}

interface ElkNode {
    id: string
    width: number
    height: number
    labels: { text: string }[]
    data: TreeNode
    parentId?: string
    chainId?: number
    x?: number
    y?: number
}

interface ElkEdge {
    id: string
    sources: string[]
    targets: string[]
    sections?: any[]
}

const elk = new ELK()
const TREE_DEPTH = 10
const HOVER_DELAY = 200

const CHAIN_COLORS = [
    '#8ab4f8', '#f28b82', '#81c995', '#fdd663', '#c58af9',
    '#ff8a65', '#4db6ac', '#e0e0e0',
]

const buildUrl = (root: string) =>
    `/api/call-tree?root=${encodeURIComponent(root)}&depth=${TREE_DEPTH}&direction=callees`

function buildGraph(data: TreeNode | TreeNode[]) {
    const nodesMap = new Map<string, TreeNode>()
    const edges: { from: string; to: string }[] = []
    const getId = (n: TreeNode) => `${n.name}@${n.file ?? 'unknown'}:${n.line}`

    const walk = (node: TreeNode) => {
        const id = getId(node)
        if (!nodesMap.has(id)) nodesMap.set(id, node)
        node.children?.forEach(child => {
            edges.push({ from: id, to: getId(child) })
            walk(child)
        })
    }

    if (Array.isArray(data)) data.forEach(walk)
    else walk(data)

    return {
        nodes: Array.from(nodesMap.entries()).map(([id, data]) => ({ id, data })),
        edges,
    }
}

function computeChains(nodes: { id: string }[], edges: { from: string; to: string }[]) {
    const adj = new Map<string, Set<string>>()
    nodes.forEach(n => adj.set(n.id, new Set()))
    edges.forEach(e => {
        adj.get(e.from)?.add(e.to)
        adj.get(e.to)?.add(e.from)
    })

    const visited = new Set<string>()
    const chainMap = new Map<string, number>()
    let chainId = 0

    nodes.forEach(n => {
        if (!visited.has(n.id)) {
            const stack = [n.id]
            while (stack.length) {
                const id = stack.pop()!
                if (visited.has(id)) continue
                visited.add(id)
                chainMap.set(id, chainId)
                adj.get(id)?.forEach(nb => { if (!visited.has(nb)) stack.push(nb) })
            }
            chainId++
        }
    })
    return chainMap
}

const TreeViewer: React.FC<{ focus?: string | null }> = ({ focus }) => {
    const [layout, setLayout] = useState<{
        children?: ElkNode[]
        edges?: ElkEdge[]
        chainMap: Map<string, number>
    } | null>(null)
    const [collapsed, setCollapsed] = useState<Set<string>>(new Set())
    const [hoverChain, setHoverChain] = useState<number | null>(null)
    const [actualHoverChain, setActualHoverChain] = useState<number | null>(null)
    const [search, setSearch] = useState('')
    const [scale, setScale] = useState(1)
    const [offset, setOffset] = useState({ x: 0, y: 0 })
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState<string | null>(null)
    const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)

    const svgRef = useRef<SVGSVGElement>(null)
    const isDragging = useRef(false)
    const lastPos = useRef({ x: 0, y: 0 })
    const hoverTimer = useRef<number | null>(null)
    const rootInput = focus || '*'

    const fetchAndLayout = useCallback(async () => {
        setLoading(true)
        setError(null)
        try {
            const resp = await fetch(buildUrl(rootInput))
            if (!resp.ok) throw new Error(await resp.text())
            const data = await resp.json()

            const { nodes, edges } = buildGraph(data)
            const chainMap = computeChains(nodes, edges)

            const elkNodes: ElkNode[] = nodes.map(n => ({
                id: n.id,
                width: 240,
                height: 64,
                labels: [{ text: n.data.name }],
                data: n.data,
                chainId: chainMap.get(n.id) ?? -1,
            }))

            const elkEdges: ElkEdge[] = edges.map((e, i) => ({
                id: `${e.from}->${e.to}-${i}`,
                sources: [e.from],
                targets: [e.to],
            }))

            const graph = {
                id: 'root',
                layoutOptions: {
                    'elk.algorithm': 'layered',
                    'elk.direction': 'RIGHT',
                    'elk.spacing.nodeNode': '120',
                    'elk.layered.spacing.nodeNodeBetweenLayers': '180',
                },
                children: elkNodes,
                edges: elkEdges,
            }

            const result = await elk.layout(graph)
            setLayout({
                children: result.children as ElkNode[],
                edges: result.edges as ElkEdge[],
                chainMap,
            })
            setCollapsed(new Set())
        } catch (e: any) {
            setError(e.message)
        } finally {
            setLoading(false)
        }
    }, [rootInput])

    useEffect(() => {
        fetchAndLayout()
    }, [fetchAndLayout])

    const onChainMouseEnter = useCallback((chainId: number) => {
        if (hoverTimer.current) window.clearTimeout(hoverTimer.current)
        setHoverChain(chainId)
        hoverTimer.current = window.setTimeout(() => {
            setActualHoverChain(chainId)
        }, HOVER_DELAY)
    }, [])

    const onChainMouseLeave = useCallback(() => {
        if (hoverTimer.current) {
            window.clearTimeout(hoverTimer.current)
            hoverTimer.current = null
        }
        setHoverChain(null)
        setActualHoverChain(null)
    }, [])

    const handleWheel = (e: React.WheelEvent) => {
        e.preventDefault()
        const delta = e.deltaY > 0 ? 0.9 : 1.1
        setScale(s => Math.max(0.2, Math.min(2, s * delta)))
    }

    const handleMouseDown = (e: React.MouseEvent) => {
        isDragging.current = true
        lastPos.current = { x: e.clientX, y: e.clientY }
    }

    const handleMouseMove = (e: React.MouseEvent) => {
        if (!isDragging.current) return
        const dx = e.clientX - lastPos.current.x
        const dy = e.clientY - lastPos.current.y
        setOffset(o => ({ x: o.x + dx, y: o.y + dy }))
        lastPos.current = { x: e.clientX, y: e.clientY }
    }

    const handleMouseUp = () => { isDragging.current = false }

    const toggleCollapse = (nodeId: string) => {
        setCollapsed(prev => {
            const next = new Set(prev)
            if (next.has(nodeId)) next.delete(nodeId)
            else next.add(nodeId)
            return next
        })
    }

    const centerOnNode = (nodeId: string) => {
        const node = layout?.children?.find(n => n.id === nodeId)
        if (!node || node.x === undefined || node.y === undefined) return
        const svgEl = svgRef.current
        if (!svgEl) return
        const rect = svgEl.getBoundingClientRect()
        setOffset({
            x: rect.width / 2 - node.x * scale,
            y: rect.height / 2 - node.y * scale,
        })
    }

    const isNodeVisible = (nodeId: string): boolean => {
        let current = nodeId
        while (current) {
            if (collapsed.has(current)) return false
            const parent = layout?.children?.find(n => n.id === current)
            current = parent?.parentId || ''
            if (!current) break
        }
        return true
    }

    if (loading) return <div style={{ color: '#888', padding: 20 }}>Loading...</div>
    if (error) return <div style={{ color: '#c44', padding: 20 }}>{error}</div>
    if (!layout) return null

    const visibleNodes = layout.children?.filter(n => isNodeVisible(n.id!)) || []
    const visibleEdges = layout.edges?.filter(e =>
        isNodeVisible(e.sources[0]) && isNodeVisible(e.targets[0])
    ) || []

    const searchResults = search
        ? visibleNodes.filter(n => n.data.name.toLowerCase().includes(search.toLowerCase()))
        : []

    const edgePoints = (edge: ElkEdge) => {
        const source = layout.children?.find(n => n.id === edge.sources[0])
        const target = layout.children?.find(n => n.id === edge.targets[0])
        if (!source || !target || source.x === undefined || source.y === undefined ||
            target.x === undefined || target.y === undefined) {
            return { sx: 0, sy: 0, tx: 0, ty: 0 }
        }
        const sx = source.x + source.width
        const sy = source.y + source.height / 2
        const tx = target.x
        const ty = target.y + target.height / 2
        return { sx, sy, tx, ty }
    }

    return (
        <div style={{ width: '100%', height: '90vh', background: '#0b0f14', overflow: 'hidden', position: 'relative' }}>
            <div style={{
                position: 'absolute', top: 10, left: 10, zIndex: 10,
                display: 'flex', gap: 8, alignItems: 'center', flexWrap: 'wrap'
            }}>
                <input
                    type="text"
                    placeholder="Search function..."
                    value={search}
                    onChange={e => setSearch(e.target.value)}
                    style={{
                        background: '#1e1e2e', border: '1px solid #444', color: '#ccc',
                        padding: '4px 8px', borderRadius: 4, width: 180
                    }}
                    onKeyDown={e => {
                        if (e.key === 'Enter' && searchResults.length > 0) {
                            centerOnNode(searchResults[0].id!)
                            setSelectedNodeId(searchResults[0].id!)
                        }
                    }}
                />
                {searchResults.length > 0 && (
                    <select
                        onChange={e => {
                            centerOnNode(e.target.value)
                            setSelectedNodeId(e.target.value)
                        }}
                        style={{
                            background: '#1e1e2e', border: '1px solid #444', color: '#ccc',
                            padding: '4px 8px', borderRadius: 4
                        }}
                    >
                        {searchResults.map(n => (
                            <option key={n.id} value={n.id}>{n.data.name}</option>
                        ))}
                    </select>
                )}
            </div>

            <div
                style={{ width: '100%', height: '100%', cursor: 'grab' }}
                onWheel={handleWheel}
                onMouseDown={handleMouseDown}
                onMouseMove={handleMouseMove}
                onMouseUp={handleMouseUp}
            >
                <svg ref={svgRef} width="100%" height="100%">
                    <defs>
                        {CHAIN_COLORS.map((color, i) => (
                            <marker
                                key={`arrow-${i}`}
                                id={`arrow-${i}`}
                                viewBox="0 0 10 10"
                                refX="9"
                                refY="5"
                                markerWidth="6"
                                markerHeight="6"
                                orient="auto"
                            >
                                <path d="M 0 0 L 10 5 L 0 10 z" fill={color} opacity="0.8" />
                            </marker>
                        ))}
                    </defs>
                    <g transform={`translate(${offset.x},${offset.y}) scale(${scale})`}>
                        {visibleEdges.map(edge => {
                            const chainId = layout.chainMap.get(edge.sources[0]) ?? -1
                            const color = CHAIN_COLORS[chainId % CHAIN_COLORS.length]
                            const isHovered = actualHoverChain === chainId
                            const opacity = actualHoverChain === null ? 1 : (isHovered ? 1 : 0.08)
                            const { sx, sy, tx, ty } = edgePoints(edge)

                            return (
                                <line
                                    key={edge.id}
                                    x1={sx}
                                    y1={sy}
                                    x2={tx}
                                    y2={ty}
                                    stroke={color}
                                    strokeOpacity={opacity}
                                    strokeWidth={2}
                                    markerEnd={`url(#arrow-${chainId % CHAIN_COLORS.length})`}
                                    style={{ transition: 'stroke-opacity 0.3s' }}
                                    onMouseEnter={() => onChainMouseEnter(chainId)}
                                    onMouseLeave={onChainMouseLeave}
                                />
                            )
                        })}

                        {visibleNodes.map(node => {
                            const chainId = node.chainId ?? -1
                            const color = CHAIN_COLORS[chainId % CHAIN_COLORS.length]
                            const isHovered = actualHoverChain === chainId
                            const opacity = actualHoverChain === null ? 1 : (isHovered ? 1 : 0.2)
                            const isCollapsed = collapsed.has(node.id!)
                            const isClass = node.data.kind === 'Class'
                            const rectRx = isClass ? 4 : 10

                            return (
                                <g
                                    key={node.id}
                                    transform={`translate(${node.x ?? 0}, ${node.y ?? 0})`}
                                    style={{
                                        cursor: 'pointer',
                                        transition: 'opacity 0.3s',
                                    }}
                                    onMouseEnter={() => onChainMouseEnter(chainId)}
                                    onMouseLeave={onChainMouseLeave}
                                >
                                    <rect
                                        width={node.width}
                                        height={node.height}
                                        rx={rectRx}
                                        fill="#121820"
                                        stroke={color}
                                        strokeWidth={selectedNodeId === node.id ? 3 : 2}
                                        opacity={opacity}
                                        onClick={(e) => {
                                            e.stopPropagation()
                                            toggleCollapse(node.id!)
                                        }}
                                    />
                                    <text
                                        x={node.width / 2}
                                        y={22}
                                        fill="#e6edf3"
                                        fontSize="12"
                                        textAnchor="middle"
                                    >
                                        {node.data.name}
                                        {isCollapsed ? ' ▶' : ' ▼'}
                                    </text>
                                    <text
                                        x={node.width / 2}
                                        y={42}
                                        fill="#8b949e"
                                        fontSize="10"
                                        textAnchor="middle"
                                    >
                                        {node.data.kind}:{node.data.line}
                                    </text>
                                </g>
                            )
                        })}
                    </g>
                </svg>
            </div>
        </div>
    )
}

export default TreeViewer