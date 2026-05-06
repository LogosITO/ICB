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
}

interface ElkEdge {
    id: string
    sources: string[]
    targets: string[]
}

const elk = new ELK()
const TREE_DEPTH = 10

const buildUrl = (root: string) =>
    `/api/call-tree?root=${encodeURIComponent(root)}&depth=${TREE_DEPTH}&direction=callees`

const flatten = (
    node: TreeNode,
    nodes: ElkNode[],
    edges: ElkEdge[],
    parentId?: string,
    index: number = 0
) => {
    const id = parentId ? `${parentId}/${node.name}-${index}` : `${node.name}-${index}`

    nodes.push({
        id,
        width: 220,
        height: 60,
        labels: [{ text: node.name }],
        data: node,
    })

    node.children.forEach((child, i) => {
        const childId = `${id}/${child.name}-${i}`

        edges.push({
            id: `${id}->${childId}`,
            sources: [id],
            targets: [childId],
        })

        flatten(child, nodes, edges, id, i)
    })
}

const CallGraphViewer: React.FC<{ focus?: string | null }> = ({ focus }) => {
    const [layout, setLayout] = useState<any>(null)
    const [currentRoot, setCurrentRoot] = useState<string>(focus || '*')
    const [scale, setScale] = useState(1)
    const [offset, setOffset] = useState({ x: 0, y: 0 })

    const isDragging = useRef(false)
    const lastPos = useRef({ x: 0, y: 0 })

    useEffect(() => {
        fetch(buildUrl(currentRoot))
            .then((r) => r.json())
            .then(async (data: any) => {
                const nodes: ElkNode[] = []
                const edges: ElkEdge[] = []

                if (currentRoot === '*') {
                    data.forEach((t: TreeNode, i: number) =>
                        flatten(t, nodes, edges, 'root', i)
                    )
                } else {
                    flatten(data, nodes, edges)
                }

                const graph = {
                    id: 'root',
                    layoutOptions: {
                        'elk.algorithm': 'layered',
                        'elk.direction': 'RIGHT',
                        'elk.layered.spacing.nodeNodeBetweenLayers': '200',
                        'elk.spacing.nodeNode': '120',
                        'elk.layered.nodePlacement.strategy': 'NETWORK_SIMPLEX',
                    },
                    children: nodes,
                    edges,
                }

                const result = await elk.layout(graph)
                setLayout(result)
            })
    }, [currentRoot])

    const handleWheel = (e: React.WheelEvent) => {
        e.preventDefault()
        const delta = e.deltaY > 0 ? 0.9 : 1.1
        setScale((s) => Math.max(0.2, Math.min(2, s * delta)))
    }

    const handleMouseDown = (e: React.MouseEvent) => {
        isDragging.current = true
        lastPos.current = { x: e.clientX, y: e.clientY }
    }

    const handleMouseMove = (e: React.MouseEvent) => {
        if (!isDragging.current) return
        const dx = e.clientX - lastPos.current.x
        const dy = e.clientY - lastPos.current.y
        setOffset((o) => ({ x: o.x + dx, y: o.y + dy }))
        lastPos.current = { x: e.clientX, y: e.clientY }
    }

    const handleMouseUp = () => {
        isDragging.current = false
    }

    if (!layout) {
        return <div style={{ color: '#888', padding: 20 }}>Loading...</div>
    }

    return (
        <div
            style={{
                width: '100%',
                height: '90vh',
                background: '#111',
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
                    {layout.edges?.map((edge: any) =>
                        edge.sections?.map((s: any, i: number) => (
                            <path
                                key={edge.id + i}
                                d={`
                                    M ${s.startPoint.x},${s.startPoint.y}
                                    C ${s.startPoint.x + 80},${s.startPoint.y}
                                      ${s.endPoint.x - 80},${s.endPoint.y}
                                      ${s.endPoint.x},${s.endPoint.y}
                                `}
                                stroke="#555"
                                fill="none"
                            />
                        ))
                    )}

                    {layout.children?.map((node: any) => (
                        <g
                            key={node.id}
                            transform={`translate(${node.x}, ${node.y})`}
                            onClick={() => setCurrentRoot(node.data.name)}
                            style={{ cursor: 'pointer' }}
                        >
                            <rect
                                width={node.width}
                                height={node.height}
                                rx={8}
                                fill="#1e1e1e"
                                stroke="#8ab4f8"
                            />
                            <text
                                x={node.width / 2}
                                y={node.height / 2 - 6}
                                fill="#ccc"
                                fontSize="12"
                                textAnchor="middle"
                                dominantBaseline="middle"
                                style={{ fontFamily: 'monospace' }}
                            >
                                {node.data.name}
                            </text>
                            <text
                                x={node.width / 2}
                                y={node.height / 2 + 10}
                                fill="#777"
                                fontSize="10"
                                textAnchor="middle"
                                dominantBaseline="middle"
                                style={{ fontFamily: 'monospace' }}
                            >
                                {node.data.kind}:{node.data.line}
                            </text>
                        </g>
                    ))}
                </g>
            </svg>
        </div>
    )
}

export default CallGraphViewer