// web/src/components/GraphViewer.tsx
import { useEffect, useRef, useState, useCallback } from 'react'
import Sigma from 'sigma'
import Graph from 'graphology'
import forceAtlas2 from 'graphology-layout-forceatlas2'
import { useGraph } from '../hooks/useGraph'

/* ── Grayscale palette ───────────────────────────────── */
const NODE_COLOR = '#b0b0b0'
const NODE_CYCLE = '#505050'
const NODE_DEAD = '#808080'
const EDGE_COLOR = 'rgba(180,180,180,0.18)'
const FONT = 'Inter, sans-serif'

interface NodeAttributes {
    label: string
    kind: string
    line: number
    is_cycle: boolean
    is_dead: boolean
    x: number
    y: number
    size: number
    color: string
}

interface EdgeAttributes {
    size: number
    color: string
}

interface Props {
    focus?: string | null
    onSelectNode: (name: string) => void
}

/** Render a minimal card when zoomed in or when node count is small */
function cardNodeReducer(settings: any, data: any) {
    const { size, color, label, line } = data
    const ctx = settings.context
    const fontSize = Math.max(10, size * 1.4)
    ctx.font = `${fontSize}px ${FONT}`
    ctx.fillStyle = color
    ctx.beginPath()
    ctx.roundRect(data.x - size * 3.5, data.y - size * 1.8, size * 7, size * 3.6, 6)
    ctx.fill()
    ctx.fillStyle = '#ffffff'
    ctx.font = `600 ${fontSize}px ${FONT}`
    ctx.fillText(label, data.x - size * 3.2, data.y - size * 0.2)
    ctx.font = `${fontSize * 0.7}px ${FONT}`
    ctx.fillStyle = '#cccccc'
    ctx.fillText(`line ${line}`, data.x - size * 3.2, data.y + size * 1.2)
}

export default function GraphViewer({ focus, onSelectNode }: Props) {
    const containerRef = useRef<HTMLDivElement>(null)
    const sigmaRef = useRef<Sigma<NodeAttributes, EdgeAttributes> | null>(null)
    const graphRef = useRef<Graph<NodeAttributes, EdgeAttributes> | null>(null)

    const [depth, setDepth] = useState('2')
    const [maxNodes, setMaxNodes] = useState('200')
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

    const getNodeColor = useCallback(
        (node: any, metricsActive: boolean) => {
            if (metricsActive) {
                if (node.is_cycle) return NODE_CYCLE
                if (node.is_dead) return NODE_DEAD
            }
            return NODE_COLOR
        },
        []
    )

    useEffect(() => {
        if (!data || !containerRef.current) return

        const g = new Graph<NodeAttributes, EdgeAttributes>()
        graphRef.current = g

        data.nodes.forEach((n, idx) => {
            g.addNode(idx, {
                label: n.name || '?',
                kind: n.kind,
                line: n.start_line,
                is_cycle: n.is_cycle || false,
                is_dead: n.is_dead || false,
                x: Math.random() * 100,
                y: Math.random() * 100,
                size: 6,
                color: getNodeColor(n, showCycles || showDead),
            })
        })

        data.edges.forEach(([src, tgt]) => {
            if (g.hasNode(src) && g.hasNode(tgt)) {
                g.addEdge(src, tgt, { size: 0.5, color: EDGE_COLOR })
            }
        })

        forceAtlas2.assign(g, {
            iterations: 120,
            settings: {
                slowDown: 0.1,
                gravity: 0.3,
                scalingRatio: 10,
                linLogMode: true,
            },
        })

        if (sigmaRef.current) sigmaRef.current.kill()

        const sigma = new Sigma<NodeAttributes, EdgeAttributes>(g, containerRef.current!, {
            renderLabels: false,
            minCameraRatio: 0.05,
            maxCameraRatio: 20,
            defaultEdgeColor: EDGE_COLOR,
            labelFont: FONT,
            labelColor: { color: '#cccccc' },
        })

        const useCards = data.nodes.length <= 200
        if (useCards) {
            sigma.setSetting('nodeReducer', (node, data) => cardNodeReducer(sigma, data))
        }

        sigma.on('clickNode', ({ node }) => {
            const attrs = g.getNodeAttributes(node)
            if (attrs.label && attrs.label !== '?') {
                onSelectNode(attrs.label)
            }
        })

        sigmaRef.current = sigma

        return () => {
            sigma.kill()
        }
    }, [data, getNodeColor, onSelectNode, showCycles, showDead])

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100%', width: '100%', background: '#111' }}>
            <div
                style={{
                    display: 'flex',
                    gap: '16px',
                    padding: '12px 16px',
                    borderBottom: '1px solid #333',
                    alignItems: 'center',
                    flexWrap: 'wrap',
                    background: '#1a1a1a',
                }}
            >
                {[
                    { label: 'Depth', value: depth, onChange: setDepth, options: ['1','2','3'] },
                    { label: 'Max Nodes', value: maxNodes, onChange: setMaxNodes, type: 'number' },
                ].map(ctrl => (
                    <label key={ctrl.label} style={labelStyle}>
                        {ctrl.label}
                        {ctrl.options ? (
                            <select value={ctrl.value} onChange={e => (ctrl.onChange as any)(e.target.value)}>
                                {ctrl.options.map(o => <option key={o}>{o}</option>)}
                            </select>
                        ) : (
                            <input
                                type="number"
                                value={ctrl.value}
                                onChange={e => ctrl.onChange(e.target.value)}
                                style={{ width: '70px' }}
                            />
                        )}
                    </label>
                ))}
                <label style={checkboxStyle}>
                    <input type="checkbox" checked={showCycles} onChange={e => setShowCycles(e.target.checked)} />
                    cycles
                </label>
                <label style={checkboxStyle}>
                    <input type="checkbox" checked={showDead} onChange={e => setShowDead(e.target.checked)} />
                    dead
                </label>
                {isLoading && <span style={{ color: '#888', fontSize: '13px' }}>loading…</span>}
            </div>
            <div ref={containerRef} style={{ flex: 1, width: '100%', background: '#111' }} />
        </div>
    )
}

const labelStyle: React.CSSProperties = {
    color: '#aaa',
    fontSize: '13px',
    display: 'flex',
    alignItems: 'center',
    gap: '4px'
}

const checkboxStyle: React.CSSProperties = {
    ...labelStyle,
    userSelect: 'none',
    cursor: 'pointer'
}