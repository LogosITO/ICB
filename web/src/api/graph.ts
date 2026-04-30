export interface GraphNode {
    name?: string;
    kind: string;
    start_line: number;
    is_cycle?: boolean;
    is_dead?: boolean;
}

export interface GraphData {
    nodes: GraphNode[];
    edges: [number, number, string][];
}

export interface NodeDetail {
    name: string;
    kind: string;
    line: number;
    file: string;
    callers: string[];
    callees: string[];
    is_cycle: boolean;
    is_dead: boolean;
}

const BASE = '/api';

export async function fetchGraph(params: Record<string, string>): Promise<GraphData> {
    const qs = new URLSearchParams(params).toString();
    const res = await fetch(`${BASE}/graph?${qs}`);
    if (!res.ok) throw new Error('Failed to fetch graph');
    return res.json();
}

export async function fetchNodeDetail(name: string): Promise<NodeDetail> {
    const res = await fetch(`${BASE}/node?name=${encodeURIComponent(name)}`);
    if (!res.ok) throw new Error('Failed to fetch node detail');
    return res.json();
}