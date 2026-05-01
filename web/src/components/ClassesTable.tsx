import { useClasses } from '../hooks/useGraph'

interface Props {
    onSelect: (name: string) => void
}

export default function ClassesTable({ onSelect }: Props) {
    const { data, isLoading, error } = useClasses()

    if (error) return <div style={{ color: 'var(--text-dim)', padding: '40px' }}>⚠️ Error loading classes</div>
    if (isLoading) return <div style={{ color: 'var(--text-dim)', padding: '40px' }}>Loading classes…</div>

    return (
        <div className="fade-in">
            <table>
                <thead>
                <tr>
                    <th>Name</th>
                    <th>Line</th>
                    <th>Methods</th>
                    <th>Complexity</th>
                </tr>
                </thead>
                <tbody>
                {(data ?? []).map(c => (
                    <tr key={c.name} onClick={() => onSelect(c.name)} style={{ cursor: 'pointer' }}>
                        <td style={{ color: 'var(--accent)', fontWeight: 500 }}>{c.name}</td>
                        <td style={{ color: 'var(--text-secondary)' }}>{c.line}</td>
                        <td>{c.methods}</td>
                        <td>{c.complexity}</td>
                    </tr>
                ))}
                </tbody>
            </table>
        </div>
    )
}