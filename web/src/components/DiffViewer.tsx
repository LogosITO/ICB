//! Diff viewer for the ICB dashboard.
//!
//! Compares two project paths (or cached graphs) and renders a colour‑coded
//! graph showing added (green), removed (red), and unchanged (grey) nodes
//! and edges.  Powered by sigma.js and graphology.

import { useState, useRef } from 'react'
import Sigma from 'sigma'
import Graph from 'graphology'
import forceAtlas2 from 'graphology-layout-forceatlas2'

const NODE_ADDED = '#4caf50'
const NODE_REMOVED = '#f44336'
const NODE_UNCHANGED = '#9e9e9e'
const EDGE_ADDED = 'rgba(76,175,80,0.4)'
const EDGE_REMOVED = 'rgba(244,67,54,0.4)'

export default function DiffViewer() {
    const [oldPath, setOldPath] = useState('')
    const [newPath, setNewPath] = useState('')
    const [data, setData] = useState<any>(null)
    const [loading, setLoading] = useState(false)
    const [error, setError] = useState('')
    const containerRef = useRef<HTMLDivElement>(null)
    const sigmaRef = useRef<Sigma | null>(null)

    const handleCompare = async () => {
        setLoading(true)
        setError('')
        try {
            const res = await fetch(`/api/diff?old=${encodeURIComponent(oldPath)}&new=${encodeURIComponent(newPath)}`)
            if (!res.ok) throw new Error(await res.text())
            const json = await res.json()
            setData(json)
            renderGraph(json)
        } catch (e: any) {
            setError(e.message)
        } finally {
            setLoading(false)
        }
    }

    const renderGraph = (diff: any) => {
        if (!containerRef.current) return
        if (sigmaRef.current) {
            sigmaRef.current.kill()
            sigmaRef.current = null
        }
        const g = new Graph({ multi: true })
        diff.nodes.forEach((n: any, idx: number) => {
            g.addNode(idx, {
                label: n.name,
                kind: n.kind,
                line: n.line,
                color: n.status === 'Added' ? NODE_ADDED : n.status === 'Removed' ? NODE_REMOVED : NODE_UNCHANGED,
                size: 8,
                x: Math.random() * 100,
                y: Math.random() * 100,
            })
        })
        const nameToIdx = new Map(diff.nodes.map((n: any, i: number) => [n.name, i]))
        diff.edges.forEach((e: any) => {
            const src = nameToIdx.get(e.source)
            const tgt = nameToIdx.get(e.target)
            if (src !== undefined && tgt !== undefined) {
                g.addEdge(src, tgt, {
                    color: e.status === 'Added' ? EDGE_ADDED : EDGE_REMOVED,
                })
            }
        })
        if (g.order === 0) return
        forceAtlas2.assign(g, { iterations: 100, settings: { slowDown: 0.1, gravity: 0.3, scalingRatio: 10, linLogMode: true } })
        const sigma = new Sigma(g, containerRef.current, { renderLabels: true, allowInvalidContainer: true })
        sigma.refresh()
        sigmaRef.current = sigma
    }

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: 'calc(100vh - 80px)', width: '100%', background: '#111' }}>
            <div style={{
                display: 'flex',
                gap: '1rem',
                padding: '1rem',
                background: '#1a1a1a',
                borderBottom: '1px solid #333',
                borderRadius: '8px 8px 0 0',
                boxShadow: '0 2px 8px rgba(0,0,0,0.5)',
            }}>
                <input
                    placeholder="Old project path"
                    value={oldPath}
                    onChange={e => setOldPath(e.target.value)}
                    style={{ flex: 1, padding: '0.5rem', background: '#111', border: '1px solid #333', borderRadius: '4px', color: '#ccc' }}
                />
                <input
                    placeholder="New project path"
                    value={newPath}
                    onChange={e => setNewPath(e.target.value)}
                    style={{ flex: 1, padding: '0.5rem', background: '#111', border: '1px solid #333', borderRadius: '4px', color: '#ccc' }}
                />
                <button
                    onClick={handleCompare}
                    style={{
                        padding: '0.5rem 1.5rem',
                        background: '#b0b0b0',
                        border: 'none',
                        borderRadius: '4px',
                        color: '#111',
                        fontWeight: 600,
                        cursor: 'pointer',
                        transition: 'background 0.2s',
                    }}
                >
                    Compare
                </button>
            </div>
            {loading && <div style={{ color: '#888', padding: '1rem' }}>Loading…</div>}
            {error && <div style={{ color: '#f44', padding: '1rem' }}>{error}</div>}
            <div ref={containerRef} style={{
                flex: 1,
                width: '100%',
                background: '#111',
                position: 'relative',
                borderRadius: '0 0 8px 8px',
                minHeight: '200px',
            }}>
                {data && data.nodes.length === 0 && data.edges.length === 0 && (
                    <div style={{
                        position: 'absolute',
                        top: '50%',
                        left: '50%',
                        transform: 'translate(-50%, -50%)',
                        color: '#888',
                        fontSize: '1.2rem',
                        pointerEvents: 'none',
                    }}>
                        No differences found.
                    </div>
                )}
            </div>
        </div>
    )
}