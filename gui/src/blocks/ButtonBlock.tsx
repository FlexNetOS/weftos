/**
 * Button block — action trigger.
 */

import { useCallback } from 'react';
import { registerBlock } from '../engine';
import type { BlockComponentProps } from '../engine';

const VARIANT_STYLES: Record<string, string> = {
  primary: 'bg-indigo-600 hover:bg-indigo-500 text-white',
  secondary: 'bg-gray-700 hover:bg-gray-600 text-gray-200',
  danger: 'bg-red-600 hover:bg-red-500 text-white',
};

function ButtonBlock({ element, resolvedProps }: BlockComponentProps) {
  const label = (resolvedProps.label as string) ?? 'Action';
  const variant = (resolvedProps.variant as string) ?? 'primary';
  const disabled = (resolvedProps.disabled as boolean) ?? false;

  const handlePress = useCallback(() => {
    const action = element.on?.press;
    if (!action) return;
    // In the full implementation, this dispatches through the Action Catalog.
    // For K8.1, log the action to console.
    console.log('[BlockEngine] Action dispatched:', action);
  }, [element.on]);

  const style = VARIANT_STYLES[variant] ?? VARIANT_STYLES.primary;

  return (
    <button
      onClick={handlePress}
      disabled={disabled}
      className={`px-4 py-2 text-sm font-medium rounded-lg transition-colors ${style} ${
        disabled ? 'opacity-50 cursor-not-allowed' : ''
      }`}
    >
      {label}
    </button>
  );
}

registerBlock('Button', ButtonBlock);
export default ButtonBlock;
