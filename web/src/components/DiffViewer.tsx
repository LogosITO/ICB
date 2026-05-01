import { useState } from 'react'

export default function DiffViewer() {
    const [oldData, setOldData] = useState<any>(null)
    const [newData, setNewData] = useState<any>(null)

    const handleFile = (setter: (data: any) => void) => (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0]
        if (!file) return
        const reader = new FileReader()
        reader.onload = () => {
            try { setter(JSON.parse(reader.result as string)) } catch {}
        }
        reader.readAsText(file)
    }

    return (
        <div>
            <h2 style={{ fontSize: '20px', fontWeight: 600, marginBottom: '24px', color: 'var(--accent)' }}>Diff</h2>
            <div style={{ display: 'flex', gap: '32px', marginBottom: '24px' }}>
                <label style={{ color: 'var(--text-dim)', fontSize: '13px' }}>
                    Old graph (JSON)
                    <input type="file" accept=".json" onChange={handleFile(setOldData)} style={{ marginTop: '4px', display: 'block' }} />
                </label>
                <label style={{ color: 'var(--text-dim)', fontSize: '13px' }}>
                    New graph (JSON)
                    <input type="file" accept=".json" onChange={handleFile(setNewData)} style={{ marginTop: '4px', display: 'block' }} />
                </label>
            </div>
            {oldData && newData && (
                <div style={{ color: 'var(--text)' }}>
                    <p>Old: {oldData.nodes.length} nodes, {oldData.edges.length} edges</p>
                    <p>New: {newData.nodes.length} nodes, {newData.edges.length} edges</p>
                </div>
            )}
        </div>
    )
}