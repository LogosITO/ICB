import React from 'react'

export default function Layout({
                                   sidebar,
                                   children,
                                   detail,
                               }: {
    sidebar: React.ReactNode
    children: React.ReactNode
    detail: React.ReactNode
}) {
    return (
        <div className="h-screen flex">
            <div className="w-80 bg-gray-900 border-r border-gray-700 flex flex-col p-4 gap-4 overflow-y-auto shadow-xl z-20">
                {sidebar}
            </div>
            <div className="flex-1 relative bg-gray-950">
                {children}
            </div>
            {detail && (
                <div className="w-96 bg-gray-900 border-l border-gray-700 p-4 overflow-y-auto shadow-xl z-20 transition-all">
                    {detail}
                </div>
            )}
        </div>
    )
}