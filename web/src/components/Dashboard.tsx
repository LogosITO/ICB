import { useState } from 'react'
import Overview from './Overview'
import Functions from './FunctionsTable'
import Classes from './ClassesTable'
import GraphViewer from './GraphViewer'
import DiffViewer from './DiffViewer'

type Tab = 'overview' | 'functions' | 'classes' | 'graph' | 'diff'

export default function Dashboard() {
    const [activeTab, setActiveTab] = useState<Tab>('overview')
    const [focusNode, setFocusNode] = useState<string | null>(null)

    const tabs: { key: Tab; label: string }[] = [
        { key: 'overview', label: 'Overview' },
        { key: 'functions', label: 'Functions' },
        { key: 'classes', label: 'Classes' },
        { key: 'graph', label: 'Graph' },
        { key: 'diff', label: 'Diff' },
    ]

    return (
        <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', background: '#111', color: '#ccc' }}>
            <div style={{ display: 'flex', borderBottom: '1px solid #333', background: '#1a1a1a' }}>
                {tabs.map(tab => (
                    <button
                        key={tab.key}
                        onClick={() => { setActiveTab(tab.key); setFocusNode(null); }}
                        style={{
                            padding: '0.75rem 1.5rem',
                            background: activeTab === tab.key ? '#111' : 'transparent',
                            border: 'none',
                            borderBottom: activeTab === tab.key ? '2px solid #b0b0b0' : '2px solid transparent',
                            color: activeTab === tab.key ? '#fff' : '#888',
                            cursor: 'pointer',
                            fontSize: '0.9rem',
                        }}
                    >
                        {tab.label}
                    </button>
                ))}
            </div>
            <div style={{ flex: 1, overflow: 'auto' }}>
                {activeTab === 'overview' && <Overview />}
                {activeTab === 'functions' && <Functions />}
                {activeTab === 'classes' && <Classes />}
                {activeTab === 'graph' && (
                    <GraphViewer
                        focus={focusNode}
                        onSelectNode={(name) => setFocusNode(name)}
                    />
                )}
                {activeTab === 'diff' && <DiffViewer />}
            </div>
        </div>
    )
}