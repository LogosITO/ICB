import { useQuery } from '@tanstack/react-query';
import { fetchGraph, fetchNodeDetail, fetchFunctions, fetchClasses, fetchFiles } from '../api/graph';
import type { GraphData, NodeDetail, FunctionMetric, ClassMetric, FileMetric } from '../api/graph';

export function useGraph(params: Record<string, string>) {
    return useQuery<GraphData, Error>({
        queryKey: ['graph', params],
        queryFn: () => fetchGraph(params),
        enabled: !!params.focus || !!params.kind,
        staleTime: 60_000,
        retry: 1,
    });
}

export function useNodeDetail(name: string | null) {
    return useQuery<NodeDetail, Error>({
        queryKey: ['node', name],
        queryFn: () => fetchNodeDetail(name!),
        enabled: !!name,
        retry: 1,
    });
}

export function useFunctions() {
    return useQuery<FunctionMetric[], Error>({
        queryKey: ['functions'],
        queryFn: fetchFunctions,
        staleTime: 120_000,
        retry: 1,
    });
}

export function useClasses() {
    return useQuery<ClassMetric[], Error>({
        queryKey: ['classes'],
        queryFn: fetchClasses,
        staleTime: 120_000,
        retry: 1,
    });
}

export function useFiles() {
    return useQuery<FileMetric[], Error>({
        queryKey: ['files'],
        queryFn: fetchFiles,
        staleTime: 120_000,
        retry: 1,
    });
}