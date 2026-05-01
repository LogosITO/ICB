import { useFunctions, useClasses, useFiles } from '../hooks/useGraph'

export default function Overview() {
    const { data: functions, isLoading: lf, error: ef } = useFunctions()
    const { data: classes, isLoading: lc, error: ec } = useClasses()
    const { data: files, isLoading: lfi, error: efi } = useFiles()

    if (ef || ec || efi) return <ServerError />
    if (lf || lc || lfi) return <Skeleton />

    const totalComplexity = functions?.reduce((s, f) => s + f.complexity, 0) ?? 0
    const cycles = functions?.filter(f => f.is_cycle).length ?? 0
    const dead = functions?.filter(f => f.is_dead).length ?? 0

    return (
        <div className="fade-in">
            <h2 style={headingStyle}>Project Overview</h2>
            <div style={gridStyle}>
                <MetricCard label="Functions" value={functions?.length ?? 0} />
                <MetricCard label="Classes" value={classes?.length ?? 0} />
                <MetricCard label="Files" value={files?.length ?? 0} />
                <MetricCard label="Complexity" value={totalComplexity} />
                <MetricCard label="Cycles" value={cycles} />
                <MetricCard label="Dead Code" value={dead} />
            </div>

            <h3 style={{ fontSize: '16px', fontWeight: 500, marginBottom: '16px', color: 'var(--text-secondary)' }}>
                File Complexity
            </h3>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '10px' }}>
                {(files ?? []).slice(0, 10).map(f => (
                    <div key={f.path} style={{ display: 'flex', alignItems: 'center', gap: '16px' }}>
            <span style={{ width: '200px', fontSize: '13px', color: 'var(--text)', whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' }}>
              {f.path}
            </span>
                        <div style={{ flex: 1, height: '4px', background: 'var(--border)', borderRadius: '2px', overflow: 'hidden' }}>
                            <div
                                style={{
                                    height: '100%',
                                    width: `${Math.min(100, f.functions * 5)}%`,
                                    background: 'var(--accent)',
                                    borderRadius: '2px',
                                    transition: 'width var(--transition)',
                                }}
                            />
                        </div>
                        <span style={{ fontSize: '12px', color: 'var(--text-secondary)', width: '60px', textAlign: 'right' }}>
              {f.functions} fn
            </span>
                    </div>
                ))}
            </div>
        </div>
    )
}

function MetricCard({ label, value }: { label: string; value: number }) {
    return (
        <div className="metric-card">
      <span style={{ fontSize: '12px', color: 'var(--text-secondary)', textTransform: 'uppercase', letterSpacing: '0.06em' }}>
        {label}
      </span>
            <span style={{ fontSize: '32px', fontWeight: 600, color: 'var(--accent)' }}>
        {value}
      </span>
        </div>
    )
}

function ServerError() {
    return <div style={{ color: 'var(--text-dim)', padding: '40px' }}>⚠️ Server unreachable</div>
}

function Skeleton() {
    return <div style={{ color: 'var(--text-dim)', padding: '40px' }}>Loading…</div>
}

const headingStyle: React.CSSProperties = {
    fontSize: '22px',
    fontWeight: 600,
    marginBottom: '24px',
    color: 'var(--accent)',
}

const gridStyle: React.CSSProperties = {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))',
    gap: '16px',
    marginBottom: '40px',
}