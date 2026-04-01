/**
 * ResourceTree block — hierarchical tree view with expand/collapse.
 */

import { useState, useCallback } from 'react';
import { registerBlock } from '../engine';
import type { BlockComponentProps } from '../engine';

interface TreeNode {
  name: string;
  path: string;
  isDir: boolean;
  children?: TreeNode[];
}

function TreeNodeView({ node, depth }: { node: TreeNode; depth: number }) {
  const [expanded, setExpanded] = useState(depth < 2);

  const toggle = useCallback(() => {
    if (node.isDir) setExpanded((prev) => !prev);
  }, [node.isDir]);

  const hasChildren = node.isDir && node.children && node.children.length > 0;

  return (
    <div>
      <button
        onClick={toggle}
        className={`flex items-center gap-1.5 w-full text-left px-1 py-0.5 text-sm hover:bg-gray-800/50 rounded transition-colors ${
          node.isDir ? 'text-gray-200' : 'text-gray-400'
        }`}
        style={{ paddingLeft: depth * 16 + 4 }}
      >
        <span className="text-xs w-4 text-center flex-shrink-0">
          {node.isDir ? (expanded ? '\u25BE' : '\u25B8') : '\u00B7'}
        </span>
        <span className={`font-mono ${node.isDir ? 'font-medium' : ''}`}>
          {node.name}
        </span>
      </button>
      {expanded && hasChildren && (
        <div>
          {node.children!.map((child) => (
            <TreeNodeView key={child.path} node={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
}

/** Build a mock tree from rootPath for demonstration. */
function buildMockTree(rootPath: string): TreeNode[] {
  return [
    {
      name: rootPath === '/' ? '/' : rootPath.split('/').pop() ?? rootPath,
      path: rootPath,
      isDir: true,
      children: [
        { name: 'src', path: `${rootPath}/src`, isDir: true, children: [
          { name: 'main.rs', path: `${rootPath}/src/main.rs`, isDir: false },
          { name: 'lib.rs', path: `${rootPath}/src/lib.rs`, isDir: false },
        ]},
        { name: 'tests', path: `${rootPath}/tests`, isDir: true, children: [
          { name: 'integration.rs', path: `${rootPath}/tests/integration.rs`, isDir: false },
        ]},
        { name: 'Cargo.toml', path: `${rootPath}/Cargo.toml`, isDir: false },
      ],
    },
  ];
}

function ResourceTreeBlock({ resolvedProps }: BlockComponentProps) {
  const rootPath = (resolvedProps.rootPath as string) ?? '/';
  const tree = buildMockTree(rootPath);

  return (
    <div className="border border-gray-700 rounded-lg bg-gray-900/50 p-2 min-h-[100px] overflow-y-auto max-h-[400px]">
      {tree.map((node) => (
        <TreeNodeView key={node.path} node={node} depth={0} />
      ))}
    </div>
  );
}

registerBlock('ResourceTree', ResourceTreeBlock);
export default ResourceTreeBlock;
