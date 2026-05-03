//! Interactive call graph viewer for the ICB dashboard.
//!
//! Renders a Code Property Graph using sigma.js and graphology,
//! supporting node filtering, focus expansion, and cycle / dead‑code
//! highlighting.  The graph is automatically laid out with ForceAtlas2
//! and responds to window resizes.

import { useEffect, useRef, useState } from 'react'
import Sigma from 'sigma'
import Graph from 'graphology'
import forceAtlas2 from 'graphology-layout-forceatlas2'
import { useGraph } from '../hooks/useGraph'

const NODE_COLOR = '#b0b0b0'
const NODE_CYCLE = '#505050'
const NODE_DEAD = '#808080'
const EDGE_COLOR = 'rgba(180,180,180,0.18)'

interface Props {
    focus?: string | null
    onSelectNode: (name: string) => void
}

export default function GraphViewer({ focus, onSelectNode }: Props) {
    const containerRef = useRef<HTMLDivElement>(null)
    const sigmaRef = useRef<Sigma | null>(null)
    const graphRef = useRef<Graph | null>(null)

    const [depth, setDepth] = useState('2')
    const [maxNodes, setMaxNodes] = useState('100')
    const [showCycles, setShowCycles] = useState(false)
    const [showDead, setShowDead] = useState(false)

    const queryParams: Record<string, string> = focus
        ? {
            focus,
            depth,
            max_nodes: maxNodes,
            show_cycles: String(showCycles),
            show_dead: String(showDead),
            entries: 'main',
        }
        : {
            kind: 'Function',
            max_nodes: maxNodes,
            show_cycles: String(showCycles),
            show_dead: String(showDead),
            entries: 'main',
        }

    const { data, isLoading } = useGraph(queryParams)

    useEffect(() => {
        if (!data || !containerRef.current) return

        if (sigmaRef.current) {
            sigmaRef.current.kill()
            sigmaRef.current = null
        }

        if (!data.nodes || data.nodes.length === 0) {
            graphRef.current = null
            return
        }

        const g = new Graph({ multi: true })
        graphRef.current = g

        data.nodes.forEach((n: any, idx: number) => {
            g.addNode(idx, {
                label: n.name || '?',
                kind: n.kind,
                line: n.start_line,
                is_cycle: n.is_cycle || false,
                is_dead: n.is_dead || false,
                x: Math.random() * 100,
                y: Math.random() * 100,
                size: 6,
                color:
                    (showCycles || showDead) && n.is_cycle
                        ? NODE_CYCLE
                        : (showCycles || showDead) && n.is_dead
                            ? NODE_DEAD
                            : NODE_COLOR,
            })
        })

        const edges = data.edges as [number, number, ...unknown[]][];
        edges.forEach((edge) => {
            const src = edge[0];
            const tgt = edge[1];
            if (g.hasNode(src) && g.hasNode(tgt)) {
                g.addEdge(src, tgt, { size: 0.5, color: EDGE_COLOR })
            }
        })

        forceAtlas2.assign(g, {
            iterations: 100,
            settings: {
                slowDown: 0.1,
                gravity: 0.3,
                scalingRatio: 10,
                linLogMode: true,
            },
        })

        const sigma = new Sigma(g, containerRef.current!, {
            renderLabels: false,
            minCameraRatio: 0.05,
            maxCameraRatio: 20,
            allowInvalidContainer: true,
        })

        sigma.on('clickNode', ({ node }) => {
            const attrs = g.getNodeAttributes(node) as any
            if (attrs.label && attrs.label !== '?') {
                onSelectNode(attrs.label)
            }
        })

        sigmaRef.current = sigma

        setTimeout(() => sigma.refresh(), 10)

        const resizeObserver = new ResizeObserver(() => sigma.refresh())
        resizeObserver.observe(containerRef.current)

        return () => {
            resizeObserver.disconnect()
            sigma.kill()
        }
    }, [data, onSelectNode, showCycles, showDead])

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: 'calc(100vh - 80px)', width: '100%', background: '#111' }}>
            <div style={{
                display: 'flex',
                gap: '16px',
                padding: '12px 16px',
                borderBottom: '1px solid #333',
                alignItems: 'center',
                flexWrap: 'wrap',
                background: '#1a1a1a',
                borderRadius: '8px 8px 0 0',
                boxShadow: '0 2px 8px rgba(0,0,0,0.5)',
            }}>
                <label style={{ color: '#aaa', fontSize: '13px', display: 'flex', alignItems: 'center', gap: '4px' }}>
                    Depth
                    <select value={depth} onChange={e => setDepth(e.target.value)}
                            style={{ background: '#111', color: '#ccc', border: '1px solid #333', borderRadius: '4px', padding: '4px 8px' }}>
                        <option value="1">1</option>
                        <option value="2">2</option>
                        <option value="3">3</option>
                    </select>
                </label>
                <label style={{ color: '#aaa', fontSize: '13px', display: 'flex', alignItems: 'center', gap: '4px' }}>
                    Max Nodes
                    <input type="number" value={maxNodes} onChange={e => setMaxNodes(e.target.value)}
                           style={{ width: '70px', background: '#111', color: '#ccc', border: '1px solid #333', borderRadius: '4px', padding: '4px 8px' }} />
                </label>
                <label style={{ color: '#aaa', fontSize: '13px', display: 'flex', alignItems: 'center', gap: '4px', userSelect: 'none', cursor: 'pointer' }}>
                    <input type="checkbox" checked={showCycles} onChange={e => setShowCycles(e.target.checked)} />
                    cycles
                </label>
                <label style={{ color: '#aaa', fontSize: '13px', display: 'flex', alignItems: 'center', gap: '4px', userSelect: 'none', cursor: 'pointer' }}>
                    <input type="checkbox" checked={showDead} onChange={e => setShowDead(e.target.checked)} />
                    dead
                </label>
                {isLoading && <span style={{ color: '#888', fontSize: '13px' }}>loading…</span>}
            </div>
            <div ref={containerRef} style={{ flex: 1, width: '100%', background: '#111', borderRadius: '0 0 8px 8px' }} />
        </div>
    )
}