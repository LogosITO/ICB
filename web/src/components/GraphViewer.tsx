import { useEffect, useRef, useState } from 'react'
import Sigma from 'sigma'
import Graph from 'graphology'
import FA2 from 'graphology-layout-forceatlas2'
import { useGraph } from '../hooks/useGraph'
import { GraphParams } from '../App'

interface Props {
    params: GraphParams
    onSelectNode: (name: string) => void
}

function getColor(node: any, metrics: boolean) {
    if (metrics) {
        if (node.is_cycle) return '#ff4d6d'
        if (node.is_dead) return '#6b7280'
    }
    if (node.kind === 'Function') return '#60a5fa'
    if (node.kind === 'Class') return '#fbbf24'
    return '#34d399'
}

export default function GraphViewer({ params, onSelectNode }: Props) {
    const containerRef = useRef<HTMLDivElement>(null)
    const sigmaRef = useRef<Sigma | null>(null)
    const [hover, setHover] = useState<any>(null)

    const queryParams: Record<string, string> = {
        ...(params.focus ? { focus: params.focus } : { kind: params.kind ?? 'Function' }),
        depth: params.depth,
        max_nodes: params.max_nodes,
        show_cycles: params.show_cycles,
        show_dead: params.show_dead,
        entries: params.entries,
    }

    const { data, isLoading } = useGraph(queryParams)

    useEffect(() => {
        if (!data || !containerRef.current) return

        const g = new Graph()

        // -----------------------
        // NODES
        // -----------------------
        data.nodes.forEach((n, idx) => {
            g.addNode(idx, {
                label: n.name || '?',
                kind: n.kind,
                line: n.start_line,
                is_cycle: n.is_cycle || false,
                is_dead: n.is_dead || false,

                x: Math.random(),
                y: Math.random(),

                size: Math.min(10 + (n.name?.length || 0) * 0.25, 20),

                color: getColor(n, params.show_cycles === 'true' || params.show_dead === 'true'),
            })
        })

        // -----------------------
        // EDGES
        // -----------------------
        data.edges.forEach(([src, tgt]) => {
            if (src < data.nodes.length && tgt < data.nodes.length) {
                g.addEdge(src, tgt, {
                    size: 0.5,
                    color: 'rgba(148,163,184,0.15)',
                })
            }
        })

        // -----------------------
        // LAYOUT (FIXED)
        // -----------------------
        FA2.assign(g, {
            iterations: 120,
            settings: {
                gravity: 0.3,
                scalingRatio: 10,
                slowDown: 1,
            },
        })

        // -----------------------
        // SIGMA
        // -----------------------
        if (sigmaRef.current) sigmaRef.current.kill()

        const sigma = new Sigma(g, containerRef.current, {
            renderLabels: true,
            labelFont: 'Inter',
            labelColor: { color: '#e5e7eb' },

            defaultEdgeColor: '#334155',

            labelDensity: 0.08,
            labelGridCellSize: 120,

            nodeReducer: (node, attrs) => ({
                ...attrs,
                size: attrs.size,
                color: attrs.color,
            }),
        })

        // -----------------------
        // HOVER
        // -----------------------
        sigma.on('enterNode', ({ node }) => {
            const attrs = g.getNodeAttributes(node)
            setHover(attrs)
        })

        sigma.on('leaveNode', () => setHover(null))

        // -----------------------
        // CLICK
        // -----------------------
        sigma.on('clickNode', ({ node }) => {
            const attrs = g.getNodeAttributes(node)
            if (attrs.label) onSelectNode(attrs.label)
        })

        sigmaRef.current = sigma

        return () => sigma.kill()
    }, [data])

    return (
        <div className="relative w-full h-full overflow-hidden bg-gradient-to-br from-[#0b1020] via-[#0f172a] to-[#020617]">

            {/* GRAPH */}
            <div ref={containerRef} className="w-full h-full" />

            {/* LOADING */}
            {isLoading && (
                <div className="absolute inset-0 flex items-center justify-center text-slate-400">
                    Building graph...
                </div>
            )}

            {/* 🧠 OBSIDIAN-STYLE NODE CARD */}
            {hover && (
                <div
                    className="absolute pointer-events-none"
                    style={{
                        left: 20,
                        top: 20,
                    }}
                >
                    <div className="w-72 rounded-2xl border border-white/10 bg-black/60 backdrop-blur-xl shadow-2xl p-4">

                        <div className="flex justify-between items-start">
                            <div className="text-white font-semibold text-sm">
                                {hover.label}
                            </div>

                            {hover.is_cycle && (
                                <span className="text-[10px] bg-red-500/20 text-red-300 px-2 py-0.5 rounded">
                                    CYCLE
                                </span>
                            )}
                        </div>

                        <div className="text-xs text-slate-400 mt-1">
                            {hover.kind} • line {hover.line}
                        </div>

                        <div className="mt-3 flex gap-2 flex-wrap">
                            {hover.is_dead && (
                                <span className="text-[10px] bg-slate-700 text-slate-300 px-2 py-0.5 rounded">
                                    DEAD
                                </span>
                            )}
                            <span className="text-[10px] bg-blue-500/20 text-blue-300 px-2 py-0.5 rounded">
                                FUNCTION
                            </span>
                        </div>

                    </div>
                </div>
            )}
        </div>
    )
}