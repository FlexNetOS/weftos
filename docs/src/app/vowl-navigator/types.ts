// VOWL JSON data model — compatible with WebVOWL's 8-key format.
// Ref: http://vowl.visualdataweb.org/v2/

export interface VowlJson {
  header: VowlHeader;
  namespace?: VowlNamespace[];
  metrics?: VowlMetrics;
  class: VowlClassDecl[];
  classAttribute: VowlClassAttr[];
  property: VowlPropertyDecl[];
  propertyAttribute: VowlPropertyAttr[];
}

export interface VowlHeader {
  title?: string;
  iri?: string;
  version?: string;
  description?: string;
  languages?: string[];
  baseIris?: string[];
}

export interface VowlNamespace {
  prefix: string;
  iri: string;
}

export interface VowlMetrics {
  classCount?: number;
  objectPropertyCount?: number;
  datatypePropertyCount?: number;
  individualCount?: number;
}

export interface VowlClassDecl {
  id: string;
  type: string; // e.g. "owl:Class", "rdfs:Datatype", "owl:Thing"
}

export interface VowlClassAttr {
  id: string;
  iri?: string;
  baseIri?: string;
  label?: Record<string, string>; // lang -> label
  instances?: number;
  subClasses?: string[];
  superClasses?: string[];
  annotations?: Record<string, string>;
  comment?: Record<string, string>;
  attributes?: string[]; // e.g. "deprecated", "external"
  description?: string;
}

export interface VowlPropertyDecl {
  id: string;
  type: string; // e.g. "owl:ObjectProperty", "owl:DatatypeProperty"
}

export interface VowlPropertyAttr {
  id: string;
  iri?: string;
  baseIri?: string;
  domain: string; // source class id
  range: string;  // target class id
  label?: Record<string, string>;
  comment?: Record<string, string>;
  attributes?: string[];
  superproperty?: string[];
  subproperty?: string[];
  cardinality?: string;
  minCardinality?: string;
  maxCardinality?: string;
}

// Runtime layout types used by the force simulation.

export interface GraphNode {
  id: string;
  label: string;
  type: string;
  radius: number;
  instances?: number;
  attributes: string[];
  description?: string;
  iri?: string;
  // d3 force mutable fields
  x?: number;
  y?: number;
  fx?: number | null;
  fy?: number | null;
  vx?: number;
  vy?: number;
}

export interface GraphEdge {
  id: string;
  source: string | GraphNode;
  target: string | GraphNode;
  label: string;
  type: string;
  attributes: string[];
}

export type NodeShape = 'circle' | 'rect' | 'diamond';

export interface VowlStyle {
  fill: string;
  stroke: string;
  shape: NodeShape;
  dashed: boolean;
  textColor: string;
}

// Visual encoding rules per OWL type (VOWL spec).
export function vowlStyle(type: string, attributes: string[]): VowlStyle {
  const deprecated = attributes.includes('deprecated');
  const external = attributes.includes('external');

  const base: VowlStyle = (() => {
    switch (type) {
      case 'owl:Class':
        return { fill: '#aaccff', stroke: '#306998', shape: 'circle' as const, dashed: false, textColor: '#1a1a2e' };
      case 'owl:equivalentClass':
        return { fill: '#aaccff', stroke: '#306998', shape: 'circle' as const, dashed: true, textColor: '#1a1a2e' };
      case 'rdfs:Datatype':
      case 'rdfs:Literal':
        return { fill: '#ffcc33', stroke: '#b38f00', shape: 'rect' as const, dashed: false, textColor: '#1a1a2e' };
      case 'owl:Thing':
        return { fill: '#ffffff', stroke: '#000000', shape: 'circle' as const, dashed: true, textColor: '#1a1a2e' };
      case 'owl:Nothing':
        return { fill: '#1a1a2e', stroke: '#000000', shape: 'circle' as const, dashed: false, textColor: '#ffffff' };
      case 'owl:ObjectProperty':
        return { fill: '#aaccff', stroke: '#306998', shape: 'diamond' as const, dashed: false, textColor: '#1a1a2e' };
      case 'owl:DatatypeProperty':
        return { fill: '#99cc66', stroke: '#527a2e', shape: 'diamond' as const, dashed: false, textColor: '#1a1a2e' };
      case 'rdfs:subClassOf':
        return { fill: '#eeeeee', stroke: '#999999', shape: 'diamond' as const, dashed: false, textColor: '#333333' };
      default:
        return { fill: '#d9d9d9', stroke: '#666666', shape: 'circle' as const, dashed: false, textColor: '#1a1a2e' };
    }
  })();

  if (deprecated) {
    base.dashed = true;
    base.fill = '#cccccc';
    base.stroke = '#999999';
  }
  if (external) {
    base.fill = '#3c4a5a';
    base.stroke = '#667788';
    base.textColor = '#e0e0e0';
  }

  return base;
}
