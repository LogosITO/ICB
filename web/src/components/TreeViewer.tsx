import React, { useEffect, useRef, useState } from 'react'
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
    chainId?: number
}

interface ElkEdge {
    id: string
    sources: string[]
    targets: string[]
    chainId?: number
}

const elk = new ELK()
const TREE_DEPTH = 10

const buildUrl = (root: string) =>
    `/api/call-tree?root=${encodeURIComponent(root)}&depth=${TREE_DEPTH}&direction=callees`

/**
 * Build CLEAN graph from tree:
 * - no duplicates
 * - stable IDs
 * - proper edges
 */
function buildGraph(data: TreeNode | TreeNode[]) {
    const nodesMap = new Map<string, TreeNode>()
    const edges: { from: string; to: string }[] = []

    const getId = (n: TreeNode) => `${n.name}@${n.file ?? 'unknown'}:${n.line}`

    const walk = (node: TreeNode) => {
        const id = getId(node)

        if (!nodesMap.has(id)) {
            nodesMap.set(id, node)
        }

        node.children?.forEach((child) => {
            const childId = getId(child)
            edges.push({ from: id, to: childId })
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

/**
 * Connected components = "chains"
 */
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

    const dfs = (start: string) => {
        const stack = [start]

        while (stack.length) {
            const node = stack.pop()!
            if (visited.has(node)) continue
            visited.add(node)

            chainMap.set(node, chainId)

            adj.get(node)?.forEach(n => {
                if (!visited.has(n)) stack.push(n)
            })
        }
    }

    nodes.forEach(n => {
        if (!visited.has(n.id)) {
            dfs(n.id)
            chainId++
        }
    })

    return chainMap
}

/**
 * color palette per chain
 */
const COLORS = [
    '#8ab4f8',
    '#f28b82',
    '#81c995',
    '#fdd663',
    '#c58af9',
    '#ff8a65',
    '#4db6ac',
]

const getColor = (id: number) => COLORS[id % COLORS.length]

const CallGraphViewer: React.FC<{ focus?: string | null }> = ({ focus }) => {
    const [layout, setLayout] = useState<any>(null)
    const [currentRoot, setCurrentRoot] = useState<string>(focus || '*')
    const [scale, setScale] = useState(1)
    const [offset, setOffset] = useState({ x: 0, y: 0 })

    const isDragging = useRef(false)
    const lastPos = useRef({ x: 0, y: 0 })

    useEffect(() => {
        fetch(buildUrl(currentRoot))
            .then(r => r.json())
            .then(async (data) => {
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
                    chainId: chainMap.get(e.from),
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
                setLayout({ ...result, chainMap })
            })
    }, [currentRoot])

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

    const handleMouseUp = () => {
        isDragging.current = false
    }

    if (!layout) {
        return <div style={{ color: '#888', padding: 20 }}>Loading...</div>
    }

    const chainMap: Map<string, number> = layout.chainMap

    return (
        <div
            style={{
                width: '100%',
                height: '90vh',
                background: '#0b0f14',
                overflow: 'hidden',
                cursor: 'grab',
            }}
            onWheel={handleWheel}
            onMouseDown={handleMouseDown}
            onMouseMove={handleMouseMove}
            onMouseUp={handleMouseUp}
        >
            <svg width="100%" height="100%">
                <g transform={`translate(${offset.x},${offset.y}) scale(${scale})`}>

                    {layout.edges?.map((edge: any, i: number) =>
                        edge.sections?.map((s: any, j: number) => {
                            const color = getColor(chainMap.get(edge.id.split('->')[0]) ?? 0)

                            return (
                                <path
                                    key={`${i}-${j}`}
                                    d={`
                                        M ${s.startPoint.x},${s.startPoint.y}
                                        C ${s.startPoint.x + 80},${s.startPoint.y}
                                          ${s.endPoint.x - 80},${s.endPoint.y}
                                          ${s.endPoint.x},${s.endPoint.y}
                                    `}
                                    stroke={color}
                                    strokeOpacity={0.25}
                                    strokeWidth={2}
                                    fill="none"
                                />
                            )
                        })
                    )}

                    {layout.children?.map((node: any) => {
                        const chainId = chainMap.get(node.id) ?? -1
                        const color = chainId >= 0 ? getColor(chainId) : '#777'

                        return (
                            <g
                                key={node.id}
                                transform={`translate(${node.x}, ${node.y})`}
                                onClick={() => setCurrentRoot(node.data.name)}
                                style={{ cursor: 'pointer' }}
                            >
                                <rect
                                    width={node.width}
                                    height={node.height}
                                    rx={10}
                                    fill="#121820"
                                    stroke={color}
                                    strokeWidth={2}
                                />

                                <text
                                    x={node.width / 2}
                                    y={22}
                                    fill="#e6edf3"
                                    fontSize="12"
                                    textAnchor="middle"
                                >
                                    {node.data.name}
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
    )
}

export default CallGraphViewer