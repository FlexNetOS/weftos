import { registerBlock, ElementRenderer } from '../engine';
import type { BlockComponentProps } from '../engine';

function ColumnBlock({ element, descriptor, resolvedProps }: BlockComponentProps) {
  const gap = (resolvedProps.gap as number) ?? 8;
  const children = element.children ?? [];

  return (
    <div className="flex flex-col" style={{ gap }}>
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

registerBlock('Column', ColumnBlock);
export default ColumnBlock;
