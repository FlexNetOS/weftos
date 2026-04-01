import { registerBlock, ElementRenderer } from '../engine';
import type { BlockComponentProps } from '../engine';

function GridBlock({ element, descriptor, resolvedProps }: BlockComponentProps) {
  const columns = (resolvedProps.columns as number) ?? 2;
  const gap = (resolvedProps.gap as number) ?? 8;
  const children = element.children ?? [];

  return (
    <div
      className="grid"
      style={{
        gridTemplateColumns: `repeat(${columns}, 1fr)`,
        gap,
      }}
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

registerBlock('Grid', GridBlock);
export default GridBlock;
