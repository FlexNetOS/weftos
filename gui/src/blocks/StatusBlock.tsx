/**
 * Metric block — single-value gauge with label, unit, and threshold coloring.
 */

import { registerBlock } from '../engine';
import type { BlockComponentProps } from '../engine';

interface Threshold {
  warn?: number;
  crit?: number;
}

function getColor(value: number | undefined, threshold: Threshold | undefined): string {
  if (value === undefined || !threshold) return 'border-gray-700';
  if (threshold.crit !== undefined && value >= threshold.crit) return 'border-red-500/60';
  if (threshold.warn !== undefined && value >= threshold.warn) return 'border-amber-500/60';
  return 'border-emerald-500/40';
}

function getValueColor(value: number | undefined, threshold: Threshold | undefined): string {
  if (value === undefined || !threshold) return 'text-gray-100';
  if (threshold.crit !== undefined && value >= threshold.crit) return 'text-red-400';
  if (threshold.warn !== undefined && value >= threshold.warn) return 'text-amber-400';
  return 'text-emerald-400';
}

function MetricBlock({ resolvedProps }: BlockComponentProps) {
  const label = (resolvedProps.label as string) ?? '';
  const value = resolvedProps.value;
  const unit = (resolvedProps.unit as string) ?? '';
  const threshold = resolvedProps.threshold as Threshold | undefined;

  const numValue = typeof value === 'number' ? value : undefined;
  const displayValue = value !== undefined && value !== null ? String(value) : '--';

  return (
    <div className={`bg-gray-800/60 rounded-lg p-3 border ${getColor(numValue, threshold)}`}>
      <p className="text-xs text-gray-400 mb-1">{label}</p>
      <p className={`text-xl font-mono font-semibold ${getValueColor(numValue, threshold)}`}>
        {displayValue}
        {unit && <span className="text-sm text-gray-400 ml-1">{unit}</span>}
      </p>
    </div>
  );
}

registerBlock('Metric', MetricBlock);
export default MetricBlock;
