import { useState } from 'react'
import Layout from './components/Layout'
import Overview from './components/Overview'
import FunctionsTable from './components/FunctionsTable'
import ClassesTable from './components/ClassesTable'
import GraphViewer from './components/GraphViewer'
import DiffViewer from './components/DiffViewer'

type Tab = 'overview' | 'functions' | 'classes' | 'graph' | 'diff'

function App() {
    const [activeTab, setActiveTab] = useState<Tab>('overview')
    const [focusNode, setFocusNode] = useState<string | null>(null)

    const handleTabChange = (tab: Tab) => {
        setActiveTab(tab)
        if (tab !== 'graph') setFocusNode(null)
    }

    return (
        <Layout activeTab={activeTab} onTabChange={handleTabChange}>
            {activeTab === 'overview' && <Overview />}
            {activeTab === 'functions' && <FunctionsTable onSelect={(name) => { setFocusNode(name); setActiveTab('graph'); }} />}
            {activeTab === 'classes' && <ClassesTable onSelect={(name) => { setFocusNode(name); setActiveTab('graph'); }} />}
            {activeTab === 'graph' && (
                <GraphViewer
                    focus={focusNode}
                    onSelectNode={(name) => setFocusNode(name)}
                />
            )}
            {activeTab === 'diff' && <DiffViewer />}
        </Layout>
    )
}

export default App