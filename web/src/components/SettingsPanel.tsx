import { GraphParams } from '../App'

export default function SettingsPanel({
                                          params,
                                          onChange,
                                      }: {
    params: GraphParams
    onChange: (p: GraphParams) => void
}) {
    return (
        <div className="flex flex-col gap-3">
            <label className="text-xs text-gray-400 uppercase">Depth</label>
            <select
                className="bg-gray-800 border border-gray-600 rounded p-2 text-sm"
                value={params.depth}
                onChange={e => onChange({ ...params, depth: e.target.value })}
            >
                <option value="1">1 hop</option>
                <option value="2">2 hops</option>
                <option value="3">3 hops</option>
            </select>
            <label className="text-xs text-gray-400 uppercase">Max Nodes</label>
            <input
                type="number"
                className="bg-gray-800 border border-gray-600 rounded p-2 text-sm"
                value={params.max_nodes}
                onChange={e => onChange({ ...params, max_nodes: e.target.value })}
            />
            <div className="flex gap-4 mt-2">
                <label className="flex items-center gap-2 text-sm">
                    <input
                        type="checkbox"
                        checked={params.show_cycles === 'true'}
                        onChange={e => onChange({ ...params, show_cycles: e.target.checked ? 'true' : 'false' })}
                    />
                    Cycles
                </label>
                <label className="flex items-center gap-2 text-sm">
                    <input
                        type="checkbox"
                        checked={params.show_dead === 'true'}
                        onChange={e => onChange({ ...params, show_dead: e.target.checked ? 'true' : 'false' })}
                    />
                    Dead Code
                </label>
            </div>
        </div>
    )
}