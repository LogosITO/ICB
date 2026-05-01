import { useState, ReactNode } from 'react'
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

    const handleSelectNode = (name: string) => {
        setFocusNode(name)
        setActiveTab('graph')
    }

    return (
        <Layout activeTab={activeTab} onTabChange={handleTabChange}>
            <FadeIn key={activeTab}>
                {activeTab === 'overview' && <Overview />}
                {activeTab === 'functions' && <FunctionsTable onSelect={handleSelectNode} />}
                {activeTab === 'classes' && <ClassesTable onSelect={handleSelectNode} />}
                {activeTab === 'graph' && (
                    <GraphViewer focus={focusNode} onSelectNode={setFocusNode} />
                )}
                {activeTab === 'diff' && <DiffViewer />}
            </FadeIn>
        </Layout>
    )
}

function FadeIn({ children }: { children: ReactNode }) {
    return <div className="fade-in">{children}</div>
}

export default App