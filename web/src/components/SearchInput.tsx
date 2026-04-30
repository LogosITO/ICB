interface Props {
    value: string
    onChange: (v: string) => void
}
export default function SearchInput({ value, onChange }: Props) {
    return (
        <input
            type="text"
            className="bg-gray-800 border border-gray-600 rounded p-2 text-sm text-white placeholder-gray-400 focus:border-blue-500 outline-none"
            placeholder="Search function..."
            value={value}
            onChange={e => onChange(e.target.value)}
        />
    )
}