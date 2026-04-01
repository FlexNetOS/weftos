/**
 * DataTable block — tabular data with optional sorting.
 */

import { useState, useMemo, useCallback } from 'react';
import { registerBlock } from '../engine';
import type { BlockComponentProps } from '../engine';

interface ColumnDef {
  key: string;
  label: string;
}

function TableBlock({ resolvedProps }: BlockComponentProps) {
  const columns = (resolvedProps.columns as ColumnDef[]) ?? [];
  const rows = (resolvedProps.rows as Record<string, unknown>[]) ?? [];
  const sortable = (resolvedProps.sortable as boolean) ?? false;

  const [sortKey, setSortKey] = useState<string | null>(null);
  const [sortAsc, setSortAsc] = useState(true);
  const [selectedIdx, setSelectedIdx] = useState<number | null>(null);

  const handleSort = useCallback(
    (key: string) => {
      if (!sortable) return;
      if (sortKey === key) {
        setSortAsc((prev) => !prev);
      } else {
        setSortKey(key);
        setSortAsc(true);
      }
    },
    [sortable, sortKey],
  );

  const sorted = useMemo(() => {
    if (!sortKey) return rows;
    return [...rows].sort((a, b) => {
      const av = a[sortKey];
      const bv = b[sortKey];
      if (av === bv) return 0;
      if (av === undefined || av === null) return 1;
      if (bv === undefined || bv === null) return -1;
      const cmp = av < bv ? -1 : 1;
      return sortAsc ? cmp : -cmp;
    });
  }, [rows, sortKey, sortAsc]);

  if (columns.length === 0) {
    return <div className="text-gray-500 text-sm p-2">No columns defined</div>;
  }

  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm">
        <thead>
          <tr className="border-b border-gray-700 text-gray-400 text-left">
            {columns.map((col) => (
              <th
                key={col.key}
                className={`py-2 pr-4 ${sortable ? 'cursor-pointer hover:text-gray-200 select-none' : ''}`}
                onClick={() => handleSort(col.key)}
              >
                {col.label}
                {sortKey === col.key && (
                  <span className="ml-1 text-xs">{sortAsc ? '\u25B2' : '\u25BC'}</span>
                )}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {sorted.map((row, i) => (
            <tr
              key={i}
              onClick={() => setSelectedIdx(i)}
              className={`border-b border-gray-800 cursor-pointer transition-colors ${
                selectedIdx === i
                  ? 'bg-indigo-500/10'
                  : 'hover:bg-gray-800/50'
              }`}
            >
              {columns.map((col) => (
                <td key={col.key} className="py-1.5 pr-4 font-mono text-gray-200">
                  {row[col.key] !== undefined ? String(row[col.key]) : '--'}
                </td>
              ))}
            </tr>
          ))}
          {sorted.length === 0 && (
            <tr>
              <td colSpan={columns.length} className="py-4 text-center text-gray-500">
                No data
              </td>
            </tr>
          )}
        </tbody>
      </table>
    </div>
  );
}

registerBlock('DataTable', TableBlock);
export default TableBlock;
