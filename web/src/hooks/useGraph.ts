import { useQuery } from '@tanstack/react-query';
import { fetchGraph, fetchNodeDetail, fetchFunctions, fetchClasses, fetchFiles } from '../api/graph';

export function useGraph(params: Record<string, string>) {
    return useQuery({
        queryKey: ['graph', params],
        queryFn: () => fetchGraph(params),
        enabled: !!params.focus || !!params.kind,
        staleTime: 60_000,
    });
}

export function useNodeDetail(name: string | null) {
    return useQuery({
        queryKey: ['node', name],
        queryFn: () => fetchNodeDetail(name!),
        enabled: !!name,
    });
}

export function useFunctions() {
    return useQuery({
        queryKey: ['functions'],
        queryFn: fetchFunctions,
        staleTime: 120_000,
    });
}

export function useClasses() {
    return useQuery({
        queryKey: ['classes'],
        queryFn: fetchClasses,
        staleTime: 120_000,
    });
}

export function useFiles() {
    return useQuery({
        queryKey: ['files'],
        queryFn: fetchFiles,
        staleTime: 120_000,
    });
}