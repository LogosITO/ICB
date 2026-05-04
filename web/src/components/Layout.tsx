//! Layout component for the ICB dashboard.
//!
//! Provides a sidebar and a top bar with two ways to load a project:
//! 1. Type a server‑side path and press **Analyze**.
//! 2. Click **Upload ZIP** and select a ZIP archive of the project.
//!
//! All API calls go directly to `http://localhost:8080` to avoid CORS issues.

import { useState, useRef } from 'react'

type TabId = 'overview' | 'functions' | 'classes' | 'graph' | 'diff'

const tabs: { id: TabId; label: string }[] = [
    { id: 'overview', label: 'Overview' },
    { id: 'functions', label: 'Functions' },
    { id: 'classes', label: 'Classes' },
    { id: 'graph', label: 'Graph' },
    { id: 'diff', label: 'Diff' },
]

interface Props {
    activeTab: TabId
    onTabChange: (tab: TabId) => void
    children: React.ReactNode
}

const BACKEND = 'http://localhost:8080'

export default function Layout({ activeTab, onTabChange, children }: Props) {
    const [projectPath, setProjectPath] = useState('')
    const [loading, setLoading] = useState(false)
    const [message, setMessage] = useState('')

    const fileInputRef = useRef<HTMLInputElement>(null)
    const [uploading, setUploading] = useState(false)

    const analyze = async () => {
        if (!projectPath.trim()) return
        setLoading(true)
        setMessage('')
        try {
            const res = await fetch(`${BACKEND}/api/load`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ project: projectPath }),
            })
            if (!res.ok) throw new Error(await res.text())
            const data = await res.json()
            setMessage(`Loaded: ${data.nodes} nodes, ${data.edges} edges`)
            window.location.reload()
        } catch (e: any) {
            setMessage(`Error: ${e.message}`)
        } finally {
            setLoading(false)
        }
    }

    const handleZipChange = async (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0]
        if (!file) return

        setUploading(true)
        setMessage('')

        const formData = new FormData()
        formData.append('zip', file)

        try {
            const res = await fetch(`${BACKEND}/api/upload`, {
                method: 'POST',
                body: formData,
            })
            if (!res.ok) throw new Error(await res.text())
            const data = await res.json()
            setMessage(`Loaded: ${data.nodes} nodes, ${data.edges} edges`)
            window.location.reload()
        } catch (e: any) {
            setMessage(`Error: ${e.message}`)
        } finally {
            setUploading(false)
        }
    }

    return (
        <div style={{ display: 'flex', height: '100%' }}>
            <nav style={{
                display: 'flex',
                flexDirection: 'column',
                width: '200px',
                background: 'var(--surface)',
                borderRight: '1px solid var(--border)',
                padding: '24px 0',
                gap: '4px',
            }}>
                <div style={{ padding: '0 24px 24px', fontSize: '18px', fontWeight: 600, color: 'var(--accent)' }}>
                    ICB
                </div>
                {tabs.map(tab => (
                    <button key={tab.id} style={{
                        display: 'block',
                        width: '100%',
                        padding: '12px 24px',
                        background: activeTab === tab.id ? 'var(--surface-hover)' : 'transparent',
                        color: activeTab === tab.id ? 'var(--accent)' : 'var(--text-secondary)',
                        textAlign: 'left',
                        fontSize: '13px',
                        fontWeight: 500,
                        borderLeft: activeTab === tab.id ? '2px solid var(--accent)' : '2px solid transparent',
                        transition: 'background 0.2s, color 0.2s, border-color 0.2s',
                    }} onClick={() => onTabChange(tab.id)}>
                        {tab.label}
                    </button>
                ))}
            </nav>
            <main style={{ flex: 1, overflow: 'auto', padding: '32px 40px', background: 'var(--bg)' }}>
                <div style={{ marginBottom: '24px', display: 'flex', gap: '12px', alignItems: 'center', flexWrap: 'wrap' }}>
                    <input
                        type="text"
                        placeholder="Project path (e.g. ../Vizora)"
                        value={projectPath}
                        onChange={e => setProjectPath(e.target.value)}
                        style={{ flex: 1, minWidth: '200px', padding: '8px 12px', background: 'var(--surface)', border: '1px solid var(--border)', borderRadius: '4px', color: 'var(--text)', fontSize: '14px' }}
                    />
                    <button onClick={analyze} disabled={loading}
                            style={{ padding: '8px 16px', background: loading ? '#555' : 'var(--accent)', border: 'none', borderRadius: '4px', color: '#fff', fontWeight: 600, cursor: loading ? 'not-allowed' : 'pointer' }}>
                        {loading ? 'Analyzing…' : 'Analyze'}
                    </button>

                    <span style={{ color: 'var(--text-secondary)', fontSize: '13px' }}>or</span>

                    <input type="file" ref={fileInputRef} style={{ display: 'none' }} accept=".zip" onChange={handleZipChange} />
                    <button onClick={() => fileInputRef.current?.click()} disabled={uploading}
                            style={{ padding: '8px 16px', background: uploading ? '#555' : '#4caf50', border: 'none', borderRadius: '4px', color: '#fff', fontWeight: 600, cursor: uploading ? 'not-allowed' : 'pointer' }}>
                        {uploading ? 'Uploading…' : 'Upload ZIP'}
                    </button>

                    {message && <span style={{ color: 'var(--text-secondary)', fontSize: '13px', marginLeft: '8px' }}>{message}</span>}
                </div>
                {children}
            </main>
        </div>
    )
}