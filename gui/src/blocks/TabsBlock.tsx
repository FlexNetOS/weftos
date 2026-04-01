import { useState } from 'react';
import { registerBlock, ElementRenderer } from '../engine';
import type { BlockComponentProps } from '../engine';

function TabsBlock({ element, descriptor, resolvedProps }: BlockComponentProps) {
  const labels = (resolvedProps.labels as string[]) ?? [];
  const initialTab = (resolvedProps.activeTab as number) ?? 0;
  const children = element.children ?? [];
  const [active, setActive] = useState(initialTab);

  const activeChild = children[active];

  return (
    <div>
      <div className="flex border-b border-gray-700 mb-3">
        {labels.map((label, i) => (
          <button
            key={i}
            onClick={() => setActive(i)}
            className={`px-4 py-2 text-sm font-medium border-b-2 transition-colors ${
              i === active
                ? 'border-indigo-500 text-indigo-400'
                : 'border-transparent text-gray-400 hover:text-gray-200 hover:border-gray-600'
            }`}
          >
            {label}
          </button>
        ))}
      </div>
      {activeChild && (
        <ElementRenderer
          descriptor={descriptor}
          elementId={activeChild}
          depth={1}
        />
      )}
    </div>
  );
}

registerBlock('Tabs', TabsBlock);
export default TabsBlock;
