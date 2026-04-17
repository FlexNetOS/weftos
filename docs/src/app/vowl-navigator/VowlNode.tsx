'use client';

import type { GraphNode, VowlStyle } from './types';
import { vowlStyle } from './types';

interface VowlNodeProps {
  node: GraphNode;
  selected: boolean;
  dimmed: boolean;
  onSelect: (id: string) => void;
}

export function VowlNode({ node, selected, dimmed, onSelect }: VowlNodeProps) {
  const style = vowlStyle(node.type, node.attributes);
  const x = node.x ?? 0;
  const y = node.y ?? 0;
  const opacity = dimmed ? 0.2 : 1;

  return (
    <g
      transform={`translate(${x},${y})`}
      opacity={opacity}
      className="cursor-pointer"
      onClick={() => onSelect(node.id)}
    >
      <NodeShape style={style} radius={node.radius} selected={selected} />
      <text
        textAnchor="middle"
        dy="0.35em"
        fontSize={node.radius > 30 ? 11 : 9}
        fill={style.textColor}
        pointerEvents="none"
        className="select-none"
      >
        {truncateLabel(node.label, node.radius)}
      </text>
      {node.instances != null && node.instances > 0 && (
        <text
          textAnchor="middle"
          dy={node.radius + 14}
          fontSize={8}
          fill="#888"
          pointerEvents="none"
        >
          ({node.instances})
        </text>
      )}
    </g>
  );
}

function NodeShape({
  style,
  radius,
  selected,
}: {
  style: VowlStyle;
  radius: number;
  selected: boolean;
}) {
  const strokeWidth = selected ? 3 : 1.5;
  const strokeColor = selected ? '#ff6600' : style.stroke;
  const dashArray = style.dashed ? '6,3' : undefined;

  switch (style.shape) {
    case 'rect':
      return (
        <rect
          x={-radius}
          y={-radius * 0.6}
          width={radius * 2}
          height={radius * 1.2}
          rx={4}
          fill={style.fill}
          stroke={strokeColor}
          strokeWidth={strokeWidth}
          strokeDasharray={dashArray}
        />
      );
    case 'diamond': {
      const w = radius * 1.2;
      const h = radius * 0.7;
      const pts = `0,${-h} ${w},0 0,${h} ${-w},0`;
      return (
        <polygon
          points={pts}
          fill={style.fill}
          stroke={strokeColor}
          strokeWidth={strokeWidth}
          strokeDasharray={dashArray}
        />
      );
    }
    default:
      return (
        <circle
          r={radius}
          fill={style.fill}
          stroke={strokeColor}
          strokeWidth={strokeWidth}
          strokeDasharray={dashArray}
        />
      );
  }
}

function truncateLabel(label: string, radius: number): string {
  const maxChars = Math.floor(radius / 4);
  if (label.length <= maxChars) return label;
  return label.slice(0, maxChars - 1) + '\u2026';
}
