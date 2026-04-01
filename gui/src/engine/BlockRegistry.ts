/**
 * Block Registry — singleton that maps block type names to React components.
 *
 * Components register themselves at import time via `registerBlock()`.
 * The renderer looks up the component for a given block type through `getBlock()`.
 */

import type { ComponentType } from 'react';
import type { BlockElement, BlockDescriptor } from './types';

// ---------------------------------------------------------------------------
// Shared props interface that every block component receives
// ---------------------------------------------------------------------------

export interface BlockComponentProps {
  /** The element id within the descriptor */
  elementId: string;
  /** The element definition */
  element: BlockElement;
  /** Full descriptor for child resolution */
  descriptor: BlockDescriptor;
  /** Resolved props (after $state substitution) */
  resolvedProps: Record<string, unknown>;
}

// ---------------------------------------------------------------------------
// Registry singleton
// ---------------------------------------------------------------------------

type BlockComponent = ComponentType<BlockComponentProps>;

const registry = new Map<string, BlockComponent>();

/**
 * Register a React component for a given block type name.
 * Typically called at module scope so registration happens at import time.
 */
export function registerBlock(type: string, component: BlockComponent): void {
  registry.set(type, component);
}

/**
 * Retrieve the component registered for a block type, or undefined.
 */
export function getBlock(type: string): BlockComponent | undefined {
  return registry.get(type);
}

/**
 * List all registered block type names.
 */
export function listBlocks(): string[] {
  return Array.from(registry.keys());
}

/**
 * Check whether a block type has been registered.
 */
export function hasBlock(type: string): boolean {
  return registry.has(type);
}
