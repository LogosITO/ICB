import { useEffect, useRef, useState, useCallback } from 'react'
import Sigma from 'sigma'
import Graph from 'graphology'
import forceAtlas2 from 'graphology-layout-forceatlas2'
import { useGraph } from '../hooks/useGraph'
import { GraphParams } from '../App'

const COMPONENT_COLORS = [
    '#60a5fa', '#f472b6', '#34d399', '#facc15',
    '#a78bfa', '#22d3ee', '#fb923c', '#f87171',
    '#4ade80', '#c084fc'
]
const CARD_ZOOM_THRESHOLD = 0.35
const LABEL_ZOOM_THRESHOLD = 0.8
const MIN_ZOOM = 0.05
const MAX_ZOOM = 10

interface GraphNode {
    name?: string
    kind: string
    start_line: number
    is_cycle?: boolean
    is_dead?: boolean
}

interface GraphData {
    nodes: GraphNode[]
    edges: [number, number, string][]
}

interface NodeAttrs {
    label: string
    kind: string
    line: number
    isCycle: boolean
    isDead: boolean
    x: number
    y: number
    size: number
    color: string
    originalColor: string
}

function getComponents(n: number, edges: [number, number][]) {
    const parent = Array.from({ length: n }, (_, i) => i)
    const find = (x: number): number => parent[x] === x ? x : (parent[x] = find(parent[x]))
    const union = (a: number, b: number) => { parent[find(a)] = find(b) }
    edges.forEach(([a, b]) => union(a, b))
    const map = new Map<number, number>()
    let compId = 0
    return parent.map(i => {
        const r = find(i)
        if (!map.has(r)) map.set(r, compId++)
        return map.get(r)!
    })
}

export function GraphViewer({params, onSelectNode}: {
    params: GraphParams
    onSelectNode: (name: string) => void
}) {
    const containerRef = useRef<HTMLDivElement>(null)
    const sigmaRef = useRef<Sigma<NodeAttrs> | null>(null)
    const graphRef = useRef<Graph<NodeAttrs> | null>(null)
    const [zoom, setZoom] = useState(1)
    const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null)
    const [selectedNodeId, setSelectedNodeId] = useState<string | null>(null)
    const [cardScreenPos, setCardScreenPos] = useState<{ x: number; y: number } | null>(null)
    const [showCards, setShowCards] = useState(false)

    const queryParams: Record<string, string> = {
        ...(params.focus ? {focus: params.focus} : {kind: params.kind ?? 'Function'}),
        depth: params.depth,
        max_nodes: params.max_nodes,
        show_cycles: params.show_cycles,
        show_dead: params.show_dead,
        entries: params.entries,
    }

    const {data, isLoading} = useGraph(queryParams)

    const getNodeScreenPosition = useCallback((nodeId: string) => {
        if (!sigmaRef.current || !graphRef.current) return {x: 0, y: 0}
        const sigma = sigmaRef.current
        const g = graphRef.current
        const attrs = g.getNodeAttributes(nodeId)
        const viewportPos = sigma.framedGraphToViewport({x: attrs.x, y: attrs.y})
        const rect = containerRef.current!.getBoundingClientRect()
        return {
            x: rect.left + viewportPos.x,
            y: rect.top + viewportPos.y,
        }
    }, [])

    const handleHoverEnter = useCallback((nodeId: string) => {
        setHoveredNodeId(nodeId)
        setCardScreenPos(getNodeScreenPosition(nodeId))
    }, [getNodeScreenPosition])

    const handleHoverLeave = useCallback(() => {
        setHoveredNodeId(null)
        setCardScreenPos(null)
    }, [])

    const handleNodeClick = useCallback((nodeId: string) => {
        setSelectedNodeId(nodeId)
        setCardScreenPos(getNodeScreenPosition(nodeId))
        const attrs = graphRef.current?.getNodeAttributes(nodeId)
        if (attrs?.label && attrs.label !== '?') {
            onSelectNode(attrs.label)
        }
    }, [onSelectNode, getNodeScreenPosition])

    const handleStageClick = useCallback(() => {
        setSelectedNodeId(null)
        setCardScreenPos(null)
    }, [])

    useEffect(() => {
        if (!data || !containerRef.current) return

        const g = new Graph<NodeAttrs>()
        graphRef.current = g

        const components = getComponents(data.nodes.length, data.edges.map(e => [e[0], e[1]]))
        const angleStep = (2 * Math.PI) / data.nodes.length

        data.nodes.forEach((n, idx) => {
            const angle = idx * angleStep
            const color = COMPONENT_COLORS[components[idx] % COMPONENT_COLORS.length]
            g.addNode(idx, {
                label: n.name || '?',
                kind: n.kind,
                line: n.start_line || 0,
                isCycle: n.is_cycle || false,
                isDead: n.is_dead || false,
                x: Math.cos(angle) * 60,
                y: Math.sin(angle) * 60,
                size: 8,
                color,
                originalColor: color,
            })
        })

        data.edges.forEach(([src, tgt]) => {
            if (g.hasNode(src) && g.hasNode(tgt)) {
                g.addEdge(src, tgt, {size: 0.5, color: 'rgba(255,255,255,0.08)'})
            }
        })

        forceAtlas2.assign(g, {
            iterations: 500,
            settings: {
                gravity: 0.1,
                scalingRatio: 40,
                slowDown: 5,
                linLogMode: true,
                outboundAttractionDistribution: true,
            },
        })

        if (sigmaRef.current) sigmaRef.current.kill()

        const sigma = new Sigma<NodeAttrs>(g, containerRef.current!, {
            renderLabels: false,
            minCameraRatio: MIN_ZOOM,
            maxCameraRatio: MAX_ZOOM,
            labelDensity: 0.07,
            labelFont: 'Inter, sans-serif',
            labelColor: {color: '#e0e0e0'},
        })

        const camera = sigma.getCamera()
        const updateZoom = () => {
            const currentZoom = camera.ratio
            setZoom(currentZoom)
            setShowCards(currentZoom <= CARD_ZOOM_THRESHOLD)
        }
        camera.on('updated', updateZoom)

        sigma.on('enterNode', ({node}) => handleHoverEnter(node))
        sigma.on('leaveNode', handleHoverLeave)
        sigma.on('clickNode', ({node}) => handleNodeClick(node))
        sigma.on('clickStage', handleStageClick)

        sigmaRef.current = sigma

        return () => {
            sigma.kill()
        }
    }, [data, handleHoverEnter, handleHoverLeave, handleNodeClick, handleStageClick])

    const activeNodeId = hoveredNodeId || selectedNodeId
    const activeAttrs: NodeAttrs | null = activeNodeId && graphRef.current?.hasNode(activeNodeId)
        ? graphRef.current.getNodeAttributes(activeNodeId)
        : null

    return (
        <div className="relative w-full h-full bg-[#0b0f17] overflow-hidden">
            <div ref={containerRef} className="absolute inset-0"/>

            {isLoading && (
                <div className="absolute inset-0 flex items-center justify-center bg-[#0b0f17]/80 z-30">
                    <div className="w-12 h-12 border-t-2 border-blue-500 rounded-full animate-spin"/>
                </div>
            )}

            {showCards && activeAttrs && cardScreenPos && (
                <div
                    className="absolute z-20 w-72 bg-gray-900/95 backdrop-blur-xl border border-gray-700/80 rounded-2xl p-5 shadow-2xl transition-all duration-300"
                    style={{
                        left: cardScreenPos.x - 144,
                        top: cardScreenPos.y - 130,
                        pointerEvents: 'auto',
                    }}
                >
                    <div className="flex items-start justify-between">
                        <div className="flex-1 min-w-0">
                            <h3 className="text-white font-bold text-lg truncate">{activeAttrs.label}</h3>
                            <p className="text-blue-400 text-xs uppercase tracking-wider mt-1">{activeAttrs.kind}</p>
                        </div>
                        <div className="flex gap-1 ml-2">
                            {activeAttrs.isCycle && (
                                <span
                                    className="bg-red-900/60 text-red-300 px-2 py-0.5 rounded-full text-xs font-medium">
                  🔄 Cycle
                </span>
                            )}
                            {activeAttrs.isDead && (
                                <span
                                    className="bg-gray-700/60 text-gray-300 px-2 py-0.5 rounded-full text-xs font-medium">
                  💀 Dead
                </span>
                            )}
                        </div>
                    </div>
                    <div className="mt-5 space-y-3 text-sm">
                        <div className="flex justify-between items-center">
                            <span className="text-gray-400">Line</span>
                            <span className="text-gray-200 font-mono">{activeAttrs.line}</span>
                        </div>
                        <div className="flex justify-between items-center">
                            <span className="text-gray-400">Color</span>
                            <div
                                className="w-5 h-5 rounded-full border border-gray-600"
                                style={{backgroundColor: activeAttrs.color}}
                            />
                        </div>
                        <div className="flex justify-between items-center">
                            <span className="text-gray-400">Connections</span>
                            <span className="text-gray-200">
                {graphRef.current && activeNodeId ? (graphRef.current.degree(activeNodeId) || 0) : 0}
              </span>
                        </div>
                    </div>
                    <div
                        className="absolute inset-0 rounded-2xl bg-gradient-to-br from-white/5 to-transparent pointer-events-none"/>
                </div>
            )}

            {!showCards && activeAttrs && zoom >= CARD_ZOOM_THRESHOLD && zoom < LABEL_ZOOM_THRESHOLD && cardScreenPos && (
                <div
                    className="absolute z-20 bg-black/70 text-white text-xs px-3 py-1.5 rounded-full whitespace-nowrap pointer-events-none"
                    style={{left: cardScreenPos.x + 15, top: cardScreenPos.y - 25}}
                >
                    {activeAttrs.label}
                </div>
            )}

            {zoom > LABEL_ZOOM_THRESHOLD && (
                <div
                    className="absolute bottom-5 left-5 z-20 text-xs text-gray-500 bg-black/50 px-3 py-1.5 rounded-full backdrop-blur-sm">
                    🔍 Zoom in to see details
                </div>
            )}
        </div>
    )
}