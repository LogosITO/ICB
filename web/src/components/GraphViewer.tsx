import { useEffect, useRef, useState, useCallback } from 'react'
import Sigma from 'sigma'
import Graph from 'graphology'
import forceAtlas2 from 'graphology-layout-forceatlas2'
import { useGraph } from '../hooks/useGraph'

const NODE_COLOR = '#b0b0b0'
const NODE_CYCLE = '#505050'
const NODE_DEAD = '#808080'
const EDGE_COLOR = 'rgba(180,180,180,0.18)'

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

export default function GraphViewer({ focus, onSelectNode }: Props) {
    const containerRef = useRef<HTMLDivElement>(null)
    const sigmaRef = useRef<Sigma<NodeAttributes, EdgeAttributes> | null>(null)
    const graphRef = useRef<Graph<NodeAttributes, EdgeAttributes> | null>(null)

    const [depth, setDepth] = useState('2')
    const [maxNodes, setMaxNodes] = useState('100')
    const [showCycles, setShowCycles] = useState(false)
    const [showDead, setShowDead] = useState(false)
    const [layoutRunning, setLayoutRunning] = useState(false)

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

        // Убиваем старый граф
        if (sigmaRef.current) {
            sigmaRef.current.kill()
            sigmaRef.current = null
        }

        // Если нет узлов — очищаем
        if (!data.nodes || data.nodes.length === 0) {
            graphRef.current = null
            return
        }

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

        // Запускаем раскладку в фоне, показываем индикатор
        setLayoutRunning(true)
        const runLayout = () => {
            forceAtlas2.assign(g, {
                iterations: 100,
                settings: {
                    slowDown: 0.1,
                    gravity: 0.3,
                    scalingRatio: 10,
                    linLogMode: true,
                },
            })
            setLayoutRunning(false)

            // Создаём Sigma после раскладки
            const sigma = new Sigma<NodeAttributes, EdgeAttributes>(g, containerRef.current!, {
                renderLabels: false,
                minCameraRatio: 0.05,
                maxCameraRatio: 20,
                defaultEdgeColor: EDGE_COLOR,
                labelColor: { color: '#cccccc' },
                allowInvalidContainer: true,
                // WebGL if possible, but sigma auto-detects
            })

            sigma.on('clickNode', ({ node }) => {
                const attrs = g.getNodeAttributes(node)
                if (attrs.label && attrs.label !== '?') {
                    onSelectNode(attrs.label)
                }
            })

            sigmaRef.current = sigma

            // Подгоняем камеру под весь граф
            sigma.getCamera().animatedReset({ duration: 300 })

            const resizeObserver = new ResizeObserver(() => sigma.refresh())
            resizeObserver.observe(containerRef.current!)

            return () => {
                resizeObserver.disconnect()
                sigma.kill()
            }
        }

        // Небольшой таймаут, чтобы интерфейс не замер
        const timeout = setTimeout(runLayout, 20)
        return () => clearTimeout(timeout)
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
                <label style={labelStyle}>
                    Depth
                    <select value={depth} onChange={e => setDepth(e.target.value)}>
                        <option value="1">1</option>
                        <option value="2">2</option>
                        <option value="3">3</option>
                    </select>
                </label>
                <label style={labelStyle}>
                    Max Nodes
                    <input
                        type="number"
                        value={maxNodes}
                        onChange={e => setMaxNodes(e.target.value)}
                        style={{ width: '70px' }}
                    />
                </label>
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

            <div
                ref={containerRef}
                style={{
                    flex: 1,
                    width: '100%',
                    background: '#111',
                    position: 'relative',
                }}
            >
                {/* Пустое состояние */}
                {!isLoading && data && (!data.nodes || data.nodes.length === 0) && (
                    <div style={overlayCenter}>graph is empty</div>
                )}

                {/* Layout running */}
                {layoutRunning && (
                    <div style={overlayCenter}>
                        <div style={{ color: '#aaa', fontSize: '14px' }}>computing layout…</div>
                    </div>
                )}

                {/* Предупреждение о лимите */}
                {!isLoading && data && data.nodes && data.nodes.length >= parseInt(maxNodes) && (
                    <div
                        style={{
                            position: 'absolute',
                            bottom: 8,
                            right: 12,
                            color: '#888',
                            fontSize: '12px',
                            background: 'rgba(0,0,0,0.7)',
                            padding: '2px 8px',
                            borderRadius: 4,
                        }}
                    >
                        showing {data.nodes.length} nodes (limit reached)
                    </div>
                )}
            </div>
        </div>
    )
}

const labelStyle: React.CSSProperties = {
    color: '#aaa',
    fontSize: '13px',
    display: 'flex',
    alignItems: 'center',
    gap: '4px',
}

const checkboxStyle: React.CSSProperties = {
    ...labelStyle,
    userSelect: 'none',
    cursor: 'pointer',
}

const overlayCenter: React.CSSProperties = {
    position: 'absolute',
    top: '50%',
    left: '50%',
    transform: 'translate(-50%, -50%)',
    color: '#666',
    fontSize: '16px',
    pointerEvents: 'none',
}