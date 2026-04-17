import { useEffect, useRef, useState, useCallback } from 'react';
import {
  forceSimulation,
  forceLink,
  forceManyBody,
  forceCenter,
  forceCollide,
  type Simulation,
  type SimulationNodeDatum,
  type SimulationLinkDatum,
} from 'd3-force';
import type { GraphNode, GraphEdge } from './types';

interface ForceLayoutResult {
  nodes: GraphNode[];
  edges: GraphEdge[];
  tick: number;
  reheat: () => void;
  pinNode: (id: string, x: number, y: number) => void;
  unpinNode: (id: string) => void;
}

export function useForceLayout(
  inputNodes: GraphNode[],
  inputEdges: GraphEdge[],
  width: number,
  height: number,
): ForceLayoutResult {
  const simRef = useRef<Simulation<GraphNode & SimulationNodeDatum, SimulationLinkDatum<GraphNode & SimulationNodeDatum>> | null>(null);
  const [tick, setTick] = useState(0);
  const nodesRef = useRef<GraphNode[]>([]);
  const edgesRef = useRef<GraphEdge[]>([]);

  useEffect(() => {
    const nodes = inputNodes.map((n) => ({ ...n }));
    const edges = inputEdges.map((e) => ({ ...e }));
    nodesRef.current = nodes;
    edgesRef.current = edges;

    const sim = forceSimulation<GraphNode>(nodes)
      .force(
        'link',
        forceLink<GraphNode, GraphEdge>(edges)
          .id((d) => d.id)
          .distance(140),
      )
      .force('charge', forceManyBody().strength(-400))
      .force('center', forceCenter(width / 2, height / 2))
      .force('collide', forceCollide<GraphNode>().radius((d) => d.radius + 12))
      .alphaDecay(0.02)
      .on('tick', () => {
        setTick((t) => t + 1);
      });

    simRef.current = sim;

    return () => {
      sim.stop();
    };
  }, [inputNodes, inputEdges, width, height]);

  const reheat = useCallback(() => {
    simRef.current?.alpha(0.8).restart();
  }, []);

  const pinNode = useCallback((id: string, x: number, y: number) => {
    const node = nodesRef.current.find((n) => n.id === id);
    if (node) {
      node.fx = x;
      node.fy = y;
    }
  }, []);

  const unpinNode = useCallback((id: string) => {
    const node = nodesRef.current.find((n) => n.id === id);
    if (node) {
      node.fx = null;
      node.fy = null;
    }
  }, []);

  return {
    nodes: nodesRef.current,
    edges: edgesRef.current,
    tick,
    reheat,
    pinNode,
    unpinNode,
  };
}
