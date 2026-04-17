'use client';

import type { GraphNode, GraphEdge } from './types';
import { vowlStyle } from './types';

interface VowlEdgeProps {
  edge: GraphEdge;
  dimmed: boolean;
}

export function VowlEdge({ edge, dimmed }: VowlEdgeProps) {
  const src = edge.source as GraphNode;
  const tgt = edge.target as GraphNode;
  if (!src.x || !src.y || !tgt.x || !tgt.y) return null;

  const style = vowlStyle(edge.type, edge.attributes);
  const opacity = dimmed ? 0.1 : 0.6;

  const dx = tgt.x - src.x;
  const dy = tgt.y - src.y;
  const len = Math.sqrt(dx * dx + dy * dy) || 1;

  // Shorten line to stop at target node edge.
  const tgtRadius = (tgt as GraphNode).radius ?? 24;
  const endX = tgt.x - (dx / len) * tgtRadius;
  const endY = tgt.y - (dy / len) * tgtRadius;

  // Midpoint for label.
  const mx = (src.x + endX) / 2;
  const my = (src.y + endY) / 2;

  const dashArray = style.dashed ? '4,3' : undefined;

  return (
    <g opacity={opacity}>
      <line
        x1={src.x}
        y1={src.y}
        x2={endX}
        y2={endY}
        stroke={style.stroke}
        strokeWidth={1.5}
        strokeDasharray={dashArray}
        markerEnd="url(#arrowhead)"
      />
      {edge.label && (
        <text
          x={mx}
          y={my - 6}
          textAnchor="middle"
          fontSize={8}
          fill="#888"
          pointerEvents="none"
          className="select-none"
        >
          {edge.label}
        </text>
      )}
    </g>
  );
}
