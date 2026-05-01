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

export interface FunctionMetric {
    name: string;
    kind: string;
    line: number;
    file?: string;
    complexity: number;
    is_cycle: boolean;
    is_dead: boolean;
    callers: number;
    callees: number;
}

export interface ClassMetric {
    name: string;
    line: number;
    file?: string;
    methods: number;
    complexity: number;
}

export interface FileMetric {
    path: string;
    functions: number;
    classes: number;
    total_complexity: number;
    calls: number;
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

export async function fetchFunctions(): Promise<FunctionMetric[]> {
    const res = await fetch(`${BASE}/functions`);
    if (!res.ok) throw new Error('Failed to fetch functions');
    return res.json();
}

export async function fetchClasses(): Promise<ClassMetric[]> {
    const res = await fetch(`${BASE}/classes`);
    if (!res.ok) throw new Error('Failed to fetch classes');
    return res.json();
}

export async function fetchFiles(): Promise<FileMetric[]> {
    const res = await fetch(`${BASE}/files`);
    if (!res.ok) throw new Error('Failed to fetch files');
    return res.json();
}