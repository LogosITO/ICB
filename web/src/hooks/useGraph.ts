import { useQuery } from '@tanstack/react-query';
import { fetchGraph, fetchNodeDetail, GraphData, NodeDetail } from '../api/graph';

export function useGraph(params: Record<string, string>) {
    return useQuery<GraphData>({
        queryKey: ['graph', params],
        queryFn: () => fetchGraph(params),
        enabled: !!params.focus || !!params.kind,
        staleTime: 60_000,
    });
}

export function useNodeDetail(name: string | null) {
    return useQuery<NodeDetail>({
        queryKey: ['node', name],
        queryFn: () => fetchNodeDetail(name!),
        enabled: !!name,
    });
}