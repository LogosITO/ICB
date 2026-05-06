import { useState, ReactNode } from 'react'
import Layout from './components/Layout'
import Overview from './components/Overview'
import FunctionsTable from './components/FunctionsTable'
import ClassesTable from './components/ClassesTable'
import DiffViewer from './components/DiffViewer'
import TreeViewer from './components/TreeViewer'

type Tab = 'overview' | 'functions' | 'classes' | 'tree' | 'diff'

function App() {
    const [activeTab, setActiveTab] = useState<Tab>('overview')
    const [focusNode, setFocusNode] = useState<string | null>(null)

    const handleTabChange = (tab: Tab) => {
        setActiveTab(tab)
        if (tab !== 'tree') setFocusNode(null)
    }

    const handleSelectNode = (name: string) => {
        setFocusNode(name)
        setActiveTab('tree')
    }

    return (
        <Layout activeTab={activeTab} onTabChange={handleTabChange}>
            <FadeIn key={activeTab}>
                {activeTab === 'overview' && <Overview />}
                {activeTab === 'functions' && <FunctionsTable onSelect={handleSelectNode} />}
                {activeTab === 'classes' && <ClassesTable onSelect={handleSelectNode} />}
                {activeTab === 'tree' && <TreeViewer focus={focusNode} />}
                {activeTab === 'diff' && <DiffViewer />}
            </FadeIn>
        </Layout>
    )
}

function FadeIn({ children }: { children: ReactNode }) {
    return <div className="fade-in">{children}</div>
}

export default App