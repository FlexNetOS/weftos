/**
 * BlockRenderer — recursive component that takes a BlockDescriptor and
 * renders the element tree by looking up components in the BlockRegistry.
 *
 * Unknown block types render a fallback placeholder.
 */

import { useMemo } from 'react';
import { getBlock } from './BlockRegistry';
import { useStateStore, resolveProps } from './StateStore';
import type { BlockDescriptor } from './types';

const MAX_DEPTH = 6;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

interface BlockRendererProps {
  descriptor: BlockDescriptor;
}

/**
 * Top-level renderer: resolves the root element and starts recursive rendering.
 */
export function BlockRenderer({ descriptor }: BlockRendererProps) {
  const rootId = descriptor.root;
  const rootElement = descriptor.elements[rootId];
  if (!rootElement) {
    return <div className="text-red-400 text-sm p-2">Block descriptor error: root "{rootId}" not found</div>;
  }
  return <ElementRenderer descriptor={descriptor} elementId={rootId} depth={0} />;
}

// ---------------------------------------------------------------------------
// Internal recursive renderer
// ---------------------------------------------------------------------------

interface ElementRendererProps {
  descriptor: BlockDescriptor;
  elementId: string;
  depth: number;
}

function ElementRenderer({ descriptor, elementId, depth }: ElementRendererProps) {
  const element = descriptor.elements[elementId];
  const data = useStateStore((s) => s.data);

  const resolvedProps = useMemo(
    () => resolveProps(element?.props, data),
    [element?.props, data],
  );

  if (!element) {
    return (
      <div className="text-red-400 text-xs p-1">
        Missing element: {elementId}
      </div>
    );
  }

  if (depth > MAX_DEPTH) {
    return (
      <div className="text-amber-400 text-xs p-1">
        Max nesting depth ({MAX_DEPTH}) exceeded
      </div>
    );
  }

  const Component = getBlock(element.type);

  if (!Component) {
    return <UnsupportedBlock type={element.type} elementId={elementId} />;
  }

  return (
    <Component
      elementId={elementId}
      element={element}
      descriptor={descriptor}
      resolvedProps={resolvedProps}
    />
  );
}

/**
 * Re-exported so block components can render their children.
 */
export { ElementRenderer };

// ---------------------------------------------------------------------------
// Fallback for unregistered block types
// ---------------------------------------------------------------------------

function UnsupportedBlock({ type, elementId }: { type: string; elementId: string }) {
  return (
    <div className="border border-dashed border-gray-600 rounded p-3 text-center text-sm text-gray-500">
      Unsupported block: <span className="font-mono text-gray-400">{type}</span>
      <span className="text-gray-600 ml-1">({elementId})</span>
    </div>
  );
}
