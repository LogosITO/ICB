import { useState } from 'react'
import Layout from './components/Layout'
import GraphViewer from './components/GraphViewer'
import Sidebar from './components/Sidebar'
import NodeDetailPanel from './components/NodeDetailPanel'

export interface GraphParams {
    kind?: string;
    focus?: string;
    depth: string;
    max_nodes: string;
    show_cycles: string;
    show_dead: string;
    entries: string;
}

function App() {
    const [params, setParams] = useState<GraphParams>({
        kind: 'Function',
        focus: undefined,
        depth: '2',
        max_nodes: '200',
        show_cycles: 'false',
        show_dead: 'false',
        entries: 'main',
    })
    const [selectedNode, setSelectedNode] = useState<string | null>(null)

    return (
        <Layout
            sidebar={<Sidebar params={params} onApply={setParams} onSelectNode={setSelectedNode} />}
            detail={selectedNode ? <NodeDetailPanel name={selectedNode} onClose={() => setSelectedNode(null)} /> : null}
        >
            <GraphViewer params={params} onSelectNode={setSelectedNode} />
        </Layout>
    )
}

export default App