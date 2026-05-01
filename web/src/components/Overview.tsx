import { useFunctions, useClasses, useFiles } from '../hooks/useGraph'

const cardStyle: React.CSSProperties = {
    background: 'var(--surface)',
    border: '1px solid var(--border)',
    borderRadius: 'var(--radius)',
    padding: '16px 20px',
    display: 'flex',
    flexDirection: 'column',
    gap: '4px',
}

const gridStyle: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))',
    gap: '12px',
    marginBottom: '32px',
}

export default function Overview() {
    const { data: functions, isLoading: lf } = useFunctions()
    const { data: classes, isLoading: lc } = useClasses()
    const { data: files, isLoading: lfi } = useFiles()

    if (lf || lc || lfi) return <div style={{ color: 'var(--text-dim)' }}>Loading…</div>

    const totalComplexity = functions?.reduce((s, f) => s + f.complexity, 0) ?? 0
    const cycles = functions?.filter(f => f.is_cycle).length ?? 0
    const dead = functions?.filter(f => f.is_dead).length ?? 0

    return (
        <div>
            <h2 style={{ fontSize: '20px', fontWeight: 600, marginBottom: '24px', color: 'var(--accent)' }}>Overview</h2>
            <div style={gridStyle}>
                <MetricCard label="Functions" value={functions?.length ?? 0} />
                <MetricCard label="Classes" value={classes?.length ?? 0} />
                <MetricCard label="Files" value={files?.length ?? 0} />
                <MetricCard label="Complexity" value={totalComplexity} />
                <MetricCard label="Cycles" value={cycles} accent />
                <MetricCard label="Dead Code" value={dead} accent />
            </div>
            <h3 style={{ fontSize: '16px', fontWeight: 500, marginBottom: '12px', color: 'var(--text)' }}>File Complexity</h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '6px' }}>
                {(files ?? []).slice(0, 10).map(f => (
                    <div key={f.path} style={{ display: 'flex', alignItems: 'center', gap: '12px' }}>
                        <span style={{ width: '180px', fontSize: '13px', color: 'var(--text-dim)', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>{f.path}</span>
                        <div style={{ flex: 1, height: '6px', background: 'var(--border)', borderRadius: '3px', overflow: 'hidden' }}>
                            <div style={{ height: '100%', width: `${Math.min(100, f.functions * 5)}%`, background: 'var(--accent)', borderRadius: '3px' }} />
                        </div>
                        <span style={{ fontSize: '12px', color: 'var(--text-dim)', width: '60px', textAlign: 'right' }}>{f.functions} fn</span>
                    </div>
                ))}
            </div>
        </div>
    )
}

function MetricCard({ label, value, accent }: { label: string; value: number; accent?: boolean }) {
    return (
        <div style={cardStyle}>
            <span style={{ fontSize: '12px', color: 'var(--text-dim)', textTransform: 'uppercase', letterSpacing: '0.04em' }}>{label}</span>
            <span style={{ fontSize: '28px', fontWeight: 600, color: accent ? 'var(--accent)' : 'var(--text)' }}>{value}</span>
        </div>
    )
}