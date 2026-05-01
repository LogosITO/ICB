import type { ReactNode } from 'react'

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
    children: ReactNode
}

const styles = {
    nav: {
        display: 'flex',
        flexDirection: 'column' as const,
        width: '200px',
        background: 'var(--surface)',
        borderRight: '1px solid var(--border)',
        padding: '24px 0',
        gap: '4px',
    },
    item: (active: boolean) => ({
        display: 'block',
        width: '100%',
        padding: '12px 24px',
        background: active ? 'var(--surface-hover)' : 'transparent',
        color: active ? 'var(--accent)' : 'var(--text-secondary)',
        textAlign: 'left' as const,
        fontSize: '13px',
        fontWeight: 500 as const,
        borderLeft: active ? '2px solid var(--accent)' : '2px solid transparent',
        transition: 'background var(--transition-fast), color var(--transition-fast), border-color var(--transition-fast)',
    }),
    main: {
        flex: 1,
        overflow: 'auto',
        padding: '32px 40px',
        background: 'var(--bg)',
    },
}

export default function Layout({ activeTab, onTabChange, children }: Props) {
    return (
        <div style={{ display: 'flex', height: '100%' }}>
            <nav style={styles.nav}>
                <div style={{ padding: '0 24px 24px', fontSize: '18px', fontWeight: 600, color: 'var(--accent)', letterSpacing: '-0.02em' }}>
                    ICB
                </div>
                {tabs.map(tab => (
                    <button
                        key={tab.id}
                        style={styles.item(activeTab === tab.id)}
                        onClick={() => onTabChange(tab.id)}
                    >
                        {tab.label}
                    </button>
                ))}
            </nav>
            <main style={styles.main}>{children}</main>
        </div>
    )
}