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
        width: '180px',
        background: 'var(--surface)',
        borderRight: '1px solid var(--border)',
        padding: '20px 0',
        gap: '2px',
    },
    navItem: (active: boolean) => ({
        display: 'block',
        width: '100%',
        padding: '10px 20px',
        background: active ? 'var(--accent-soft)' : 'transparent',
        color: active ? 'var(--accent)' : 'var(--text-dim)',
        border: 'none',
        textAlign: 'left' as const,
        fontSize: '13px',
        fontWeight: active ? 500 : 400,
        transition: 'background 0.15s',
    }),
    main: {
        flex: 1,
        overflow: 'auto',
        padding: '24px 32px',
    },
}

export default function Layout({ activeTab, onTabChange, children }: Props) {
    return (
        <div style={{ display: 'flex', height: '100%' }}>
            <nav style={styles.nav}>
                {tabs.map(tab => (
                    <button
                        key={tab.id}
                        style={styles.navItem(activeTab === tab.id)}
                        onClick={() => onTabChange(tab.id)}
                    >
                        {tab.label}
                    </button>
                ))}
            </nav>
            <main style={styles.main}>
                {children}
            </main>
        </div>
    )
}