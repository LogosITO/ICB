//! Layout component for the ICB dashboard.
//!
//! Provides a sidebar with navigation tabs and a single drag‑and‑drop
//! area for ZIP files.  The language is fixed to C/C++ for optimal
//! analysis with Clang.  Dropping a ZIP automatically starts the analysis.

import { useState, useCallback } from 'react'

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
    const [message, setMessage] = useState('')
    const [uploading, setUploading] = useState(false)
    const [dragOver, setDragOver] = useState(false)

    const uploadZip = async (file: File) => {
        if (!file) return
        setUploading(true)
        setMessage('')
        const formData = new FormData()
        formData.append('zip', file)
        try {
            // Always analyse as C/C++ (Clang preferred)
            const res = await fetch(`${BACKEND}/api/upload?languages=cpp`, {
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

    const handleDragOver = useCallback((e: React.DragEvent) => {
        e.preventDefault()
        setDragOver(true)
    }, [])

    const handleDragLeave = useCallback((e: React.DragEvent) => {
        e.preventDefault()
        setDragOver(false)
    }, [])

    const handleDrop = useCallback(
        (e: React.DragEvent) => {
            e.preventDefault()
            setDragOver(false)
            const file = e.dataTransfer.files?.[0]
            if (file && file.name.endsWith('.zip')) {
                uploadZip(file)
            } else {
                setMessage('Please drop a ZIP file.')
            }
        },
        []
    )

    return (
        <div style={{ display: 'flex', height: '100%' }}>
            {/* ---- sidebar ---- */}
            <nav
                style={{
                    display: 'flex',
                    flexDirection: 'column',
                    width: '200px',
                    background: 'var(--surface)',
                    borderRight: '1px solid var(--border)',
                    padding: '24px 0',
                    gap: '4px',
                }}
            >
                <div
                    style={{
                        padding: '0 24px 24px',
                        fontSize: '18px',
                        fontWeight: 600,
                        color: 'var(--accent)',
                    }}
                >
                    ICB
                </div>
                {tabs.map(tab => (
                    <button
                        key={tab.id}
                        style={{
                            display: 'block',
                            width: '100%',
                            padding: '12px 24px',
                            background:
                                activeTab === tab.id
                                    ? 'var(--surface-hover)'
                                    : 'transparent',
                            color:
                                activeTab === tab.id
                                    ? 'var(--accent)'
                                    : 'var(--text-secondary)',
                            textAlign: 'left',
                            fontSize: '13px',
                            fontWeight: 500,
                            borderLeft:
                                activeTab === tab.id
                                    ? '2px solid var(--accent)'
                                    : '2px solid transparent',
                            transition:
                                'background 0.2s, color 0.2s, border-color 0.2s',
                        }}
                        onClick={() => onTabChange(tab.id)}
                    >
                        {tab.label}
                    </button>
                ))}
            </nav>

            {/* ---- main content ---- */}
            <main
                style={{
                    flex: 1,
                    overflow: 'auto',
                    padding: '32px 40px',
                    background: 'var(--bg)',
                }}
            >
                <div style={{ marginBottom: '24px', display: 'flex', flexDirection: 'column', gap: '12px' }}>
                    {/* ----- drag-and-drop zone ----- */}
                    <div
                        onDragOver={handleDragOver}
                        onDragLeave={handleDragLeave}
                        onDrop={handleDrop}
                        style={{
                            border: `2px dashed ${dragOver ? 'var(--accent)' : 'var(--border)'}`,
                            borderRadius: '8px',
                            padding: '32px',
                            textAlign: 'center',
                            color: 'var(--text-secondary)',
                            transition: 'border-color 0.2s',
                            cursor: 'pointer',
                            background: dragOver ? 'var(--surface-hover)' : 'transparent',
                        }}
                    >
                        {uploading
                            ? 'Analyzing…'
                            : dragOver
                                ? 'Drop your ZIP here'
                                : 'Drop a C/C++ project ZIP here'}
                    </div>

                    {message && (
                        <span style={{ color: 'var(--text-secondary)', fontSize: '13px' }}>
                            {message}
                        </span>
                    )}
                </div>

                {children}
            </main>
        </div>
    )
}