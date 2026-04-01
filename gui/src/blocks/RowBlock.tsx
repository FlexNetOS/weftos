import { registerBlock, ElementRenderer } from '../engine';
import type { BlockComponentProps } from '../engine';

function RowBlock({ element, descriptor, resolvedProps }: BlockComponentProps) {
  const gap = (resolvedProps.gap as number) ?? 8;
  const wrap = (resolvedProps.wrap as boolean) ?? false;
  const children = element.children ?? [];

  return (
    <div
      className={`flex flex-row ${wrap ? 'flex-wrap' : ''}`}
      style={{ gap }}
    >
      {children.map((childId) => (
        <ElementRenderer
          key={childId}
          descriptor={descriptor}
          elementId={childId}
          depth={1}
        />
      ))}
    </div>
  );
}

registerBlock('Row', RowBlock);
export default RowBlock;
