'use client';

import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { zoom as d3Zoom, zoomIdentity, type ZoomTransform } from 'd3-zoom';
import { select } from 'd3-selection';
import { drag as d3Drag } from 'd3-drag';
import type { VowlJson, GraphNode, GraphEdge } from './types';
import { useForceLayout } from './use-force-layout';
import { VowlNode } from './VowlNode';
import { VowlEdge } from './VowlEdge';

interface VowlNavigatorProps {
  data: VowlJson;
  width?: number;
  height?: number;
  className?: string;
}

function parseVowl(data: VowlJson): { nodes: GraphNode[]; edges: GraphEdge[] } {
  const attrMap = new Map(data.classAttribute.map((a) => [a.id, a]));

  const nodes: GraphNode[] = data.class.map((c) => {
    const attr = attrMap.get(c.id);
    const instances = attr?.instances ?? 0;
    const baseRadius = c.type === 'rdfs:Datatype' ? 28 : 36;
    const radius = Math.min(baseRadius + Math.sqrt(instances) * 4, 60);
    return {
      id: c.id,
      label: attr?.label?.en ?? c.id,
      type: c.type,
      radius,
      instances: instances || undefined,
      attributes: attr?.attributes ?? [],
      description: attr?.description,
      iri: attr?.iri,
    };
  });

  const propAttrMap = new Map(data.propertyAttribute.map((a) => [a.id, a]));

  const edges: GraphEdge[] = data.property.map((p) => {
    const attr = propAttrMap.get(p.id);
    return {
      id: p.id,
      source: attr?.domain ?? '',
      target: attr?.range ?? '',
      label: attr?.label?.en ?? '',
      type: p.type,
      attributes: attr?.attributes ?? [],
    };
  }).filter((e) => e.source && e.target);

  return { nodes, edges };
}

export function VowlNavigator({
  data,
  width = 900,
  height = 600,
  className = '',
}: VowlNavigatorProps) {
  const svgRef = useRef<SVGSVGElement>(null);
  const gRef = useRef<SVGGElement>(null);
  const [transform, setTransform] = useState<ZoomTransform>(zoomIdentity);
  const [selected, setSelected] = useState<string | null>(null);
  const [search, setSearch] = useState('');
  const [showDatatypes, setShowDatatypes] = useState(true);
  const [degreeFilter, setDegreeFilter] = useState(0);

  const { nodes: parsedNodes, edges: parsedEdges } = useMemo(() => parseVowl(data), [data]);
  const { nodes, edges, tick, reheat, pinNode, unpinNode } = useForceLayout(parsedNodes, parsedEdges, width, height);

  // Degree map for filtering.
  const degreeMap = useMemo(() => {
    const m = new Map<string, number>();
    for (const n of nodes) m.set(n.id, 0);
    for (const e of edges) {
      const sid = typeof e.source === 'string' ? e.source : e.source.id;
      const tid = typeof e.target === 'string' ? e.target : e.target.id;
      m.set(sid, (m.get(sid) ?? 0) + 1);
      m.set(tid, (m.get(tid) ?? 0) + 1);
    }
    return m;
  }, [nodes, edges]);

  // Zoom setup.
  useEffect(() => {
    if (!svgRef.current) return;
    const svg = select(svgRef.current);
    const zoomBehavior = d3Zoom<SVGSVGElement, unknown>()
      .scaleExtent([0.2, 5])
      .on('zoom', (event) => {
        setTransform(event.transform);
      });
    svg.call(zoomBehavior);
    return () => { svg.on('.zoom', null); };
  }, []);

  // Drag setup.
  useEffect(() => {
    if (!gRef.current) return;
    const g = select(gRef.current);
    const dragBehavior = d3Drag<SVGGElement, GraphNode>()
      .on('start', (event, d) => {
        pinNode(d.id, event.x, event.y);
        reheat();
      })
      .on('drag', (event, d) => {
        pinNode(d.id, event.x, event.y);
      })
      .on('end', (_event, d) => {
        unpinNode(d.id);
      });

    g.selectAll<SVGGElement, GraphNode>('.vowl-node')
      .data(nodes, (d) => d.id)
      .call(dragBehavior);
  }, [nodes, tick, pinNode, unpinNode, reheat]);

  const onSelect = useCallback((id: string) => {
    setSelected((prev) => (prev === id ? null : id));
  }, []);

  // Filter and search.
  const searchLower = search.toLowerCase();
  const visibleNodes = new Set<string>();
  for (const n of nodes) {
    if (!showDatatypes && (n.type === 'rdfs:Datatype' || n.type === 'rdfs:Literal')) continue;
    if (degreeFilter > 0 && (degreeMap.get(n.id) ?? 0) < degreeFilter) continue;
    visibleNodes.add(n.id);
  }

  const matchesSearch = (n: GraphNode) =>
    !searchLower || n.label.toLowerCase().includes(searchLower);

  const selectedNode = nodes.find((n) => n.id === selected);
  const selectedAttr = selectedNode
    ? data.classAttribute.find((a) => a.id === selectedNode.id)
    : null;

  // Neighbor highlight.
  const neighborSet = useMemo(() => {
    if (!selected) return new Set<string>();
    const s = new Set<string>();
    s.add(selected);
    for (const e of edges) {
      const sid = typeof e.source === 'string' ? e.source : e.source.id;
      const tid = typeof e.target === 'string' ? e.target : e.target.id;
      if (sid === selected) s.add(tid);
      if (tid === selected) s.add(sid);
    }
    return s;
  }, [selected, edges]);

  return (
    <div className={`flex flex-col gap-3 ${className}`}>
      {/* Toolbar */}
      <div className="flex flex-wrap items-center gap-3 text-sm">
        <input
          type="text"
          placeholder="Search classes..."
          value={search}
          onChange={(e) => setSearch(e.target.value)}
          className="px-2 py-1 rounded border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-900 text-neutral-900 dark:text-neutral-100 w-48"
        />
        <label className="flex items-center gap-1 text-neutral-600 dark:text-neutral-400">
          <input
            type="checkbox"
            checked={showDatatypes}
            onChange={(e) => setShowDatatypes(e.target.checked)}
          />
          Datatypes
        </label>
        <label className="flex items-center gap-1 text-neutral-600 dark:text-neutral-400">
          Min degree:
          <input
            type="range"
            min={0}
            max={6}
            value={degreeFilter}
            onChange={(e) => setDegreeFilter(Number(e.target.value))}
            className="w-20"
          />
          <span className="w-4 text-center">{degreeFilter}</span>
        </label>
        <button
          onClick={reheat}
          className="px-2 py-1 rounded bg-neutral-200 dark:bg-neutral-800 hover:bg-neutral-300 dark:hover:bg-neutral-700 text-neutral-700 dark:text-neutral-300"
        >
          Re-layout
        </button>
        {data.header.title && (
          <span className="ml-auto text-neutral-500 dark:text-neutral-500 text-xs">
            {data.header.title} {data.header.version && `v${data.header.version}`}
          </span>
        )}
      </div>

      <div className="flex gap-3">
        {/* SVG Canvas */}
        <svg
          ref={svgRef}
          width={width}
          height={height}
          className="rounded border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-950"
          style={{ touchAction: 'none' }}
        >
          <defs>
            <marker
              id="arrowhead"
              viewBox="0 0 10 10"
              refX={10}
              refY={5}
              markerWidth={6}
              markerHeight={6}
              orient="auto-start-reverse"
            >
              <path d="M 0 0 L 10 5 L 0 10 z" fill="#888" />
            </marker>
          </defs>
          <g transform={`translate(${transform.x},${transform.y}) scale(${transform.k})`} ref={gRef}>
            {edges.map((e) => {
              const sid = typeof e.source === 'string' ? e.source : e.source.id;
              const tid = typeof e.target === 'string' ? e.target : e.target.id;
              if (!visibleNodes.has(sid) || !visibleNodes.has(tid)) return null;
              const dimmed = selected != null && !neighborSet.has(sid) && !neighborSet.has(tid);
              return <VowlEdge key={e.id} edge={e} dimmed={dimmed} />;
            })}
            {nodes.map((n) => {
              if (!visibleNodes.has(n.id)) return null;
              const dimmed =
                (selected != null && !neighborSet.has(n.id)) ||
                (searchLower.length > 0 && !matchesSearch(n));
              return (
                <VowlNode
                  key={n.id}
                  node={n}
                  selected={selected === n.id}
                  dimmed={dimmed}
                  onSelect={onSelect}
                />
              );
            })}
          </g>
        </svg>

        {/* Detail Panel */}
        {selectedNode && (
          <div className="w-64 shrink-0 rounded border border-neutral-300 dark:border-neutral-700 bg-white dark:bg-neutral-950 p-3 text-sm overflow-y-auto max-h-[600px]">
            <h3 className="font-semibold text-neutral-900 dark:text-neutral-100 mb-2">
              {selectedNode.label}
            </h3>
            <dl className="space-y-1 text-neutral-600 dark:text-neutral-400">
              <dt className="font-medium text-xs uppercase text-neutral-500">Type</dt>
              <dd>{selectedNode.type}</dd>
              {selectedNode.iri && (
                <>
                  <dt className="font-medium text-xs uppercase text-neutral-500 mt-2">IRI</dt>
                  <dd className="break-all text-xs">{selectedNode.iri}</dd>
                </>
              )}
              {selectedNode.description && (
                <>
                  <dt className="font-medium text-xs uppercase text-neutral-500 mt-2">Description</dt>
                  <dd>{selectedNode.description}</dd>
                </>
              )}
              {selectedNode.instances != null && selectedNode.instances > 0 && (
                <>
                  <dt className="font-medium text-xs uppercase text-neutral-500 mt-2">Instances</dt>
                  <dd>{selectedNode.instances}</dd>
                </>
              )}
              <dt className="font-medium text-xs uppercase text-neutral-500 mt-2">Connections</dt>
              <dd>{degreeMap.get(selectedNode.id) ?? 0}</dd>
            </dl>

            {/* Connected edges */}
            <h4 className="font-medium text-xs uppercase text-neutral-500 mt-3 mb-1">Relations</h4>
            <ul className="space-y-1 text-xs">
              {edges
                .filter((e) => {
                  const sid = typeof e.source === 'string' ? e.source : e.source.id;
                  const tid = typeof e.target === 'string' ? e.target : e.target.id;
                  return sid === selectedNode.id || tid === selectedNode.id;
                })
                .map((e) => {
                  const sid = typeof e.source === 'string' ? e.source : e.source.id;
                  const tid = typeof e.target === 'string' ? e.target : e.target.id;
                  const other = sid === selectedNode.id ? tid : sid;
                  const otherNode = nodes.find((n) => n.id === other);
                  const direction = sid === selectedNode.id ? '\u2192' : '\u2190';
                  return (
                    <li key={e.id} className="text-neutral-600 dark:text-neutral-400">
                      <span className="text-neutral-400">{direction}</span>{' '}
                      <button
                        onClick={() => onSelect(other)}
                        className="text-blue-500 dark:text-blue-400 hover:underline"
                      >
                        {otherNode?.label ?? other}
                      </button>
                      {e.label && <span className="text-neutral-500 ml-1">({e.label})</span>}
                    </li>
                  );
                })}
            </ul>

            <button
              onClick={() => setSelected(null)}
              className="mt-3 text-xs text-neutral-500 hover:text-neutral-700 dark:hover:text-neutral-300"
            >
              Clear selection
            </button>
          </div>
        )}
      </div>

      {/* Legend */}
      <div className="flex flex-wrap gap-4 text-xs text-neutral-500 dark:text-neutral-500">
        <span className="flex items-center gap-1">
          <svg width={14} height={14}><circle cx={7} cy={7} r={6} fill="#aaccff" stroke="#306998" strokeWidth={1} /></svg>
          Class
        </span>
        <span className="flex items-center gap-1">
          <svg width={14} height={14}><rect x={1} y={3} width={12} height={8} rx={2} fill="#ffcc33" stroke="#b38f00" strokeWidth={1} /></svg>
          Datatype
        </span>
        <span className="flex items-center gap-1">
          <svg width={14} height={14}><polygon points="7,1 13,7 7,13 1,7" fill="#aaccff" stroke="#306998" strokeWidth={1} /></svg>
          Object Property
        </span>
        <span className="flex items-center gap-1">
          <svg width={14} height={14}><polygon points="7,1 13,7 7,13 1,7" fill="#99cc66" stroke="#527a2e" strokeWidth={1} /></svg>
          Datatype Property
        </span>
        <span className="flex items-center gap-1">
          <svg width={14} height={14}><circle cx={7} cy={7} r={6} fill="#cccccc" stroke="#999" strokeWidth={1} strokeDasharray="3,2" /></svg>
          Deprecated
        </span>
      </div>
    </div>
  );
}
