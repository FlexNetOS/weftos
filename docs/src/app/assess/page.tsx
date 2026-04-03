'use client';

import { useCallback, useEffect, useState } from 'react';

// ── Types ──────────────────────────────────────────────────────

interface AssessmentSummary {
  total_files: number;
  lines_of_code: number;
  rust_files: number;
  typescript_files: number;
  config_files: number;
  doc_files: number;
  dependency_files: number;
  complexity_warnings: number;
  coherence_score: number;
}

interface Finding {
  severity: string;
  category: string;
  file: string;
  line: number | null;
  message: string;
}

interface AssessmentReport {
  timestamp: string;
  scope: string;
  project: string;
  files_scanned: number;
  summary: AssessmentSummary;
  findings: Finding[];
}

// ── Mock data for demo ─────────────────────────────────────────

const MOCK_REPORT: AssessmentReport = {
  timestamp: new Date().toISOString(),
  scope: 'full',
  project: 'weftos',
  files_scanned: 247,
  summary: {
    total_files: 247,
    lines_of_code: 38420,
    rust_files: 142,
    typescript_files: 61,
    config_files: 18,
    doc_files: 22,
    dependency_files: 4,
    complexity_warnings: 5,
    coherence_score: 87.3,
  },
  findings: [
    {
      severity: 'warning',
      category: 'complexity',
      file: 'crates/clawft-kernel/src/assessment.rs',
      line: null,
      message: '712 lines — consider splitting (target: <500)',
    },
    {
      severity: 'info',
      category: 'technical-debt',
      file: 'crates/clawft-core/src/memory.rs',
      line: 88,
      message: '// TODO: implement LRU eviction policy',
    },
    {
      severity: 'warning',
      category: 'complexity',
      file: 'crates/clawft-cli/src/commands/assess_cmd.rs',
      line: null,
      message: '1077 lines — consider splitting (target: <500)',
    },
    {
      severity: 'info',
      category: 'technical-debt',
      file: 'crates/clawft-services/src/delegate.rs',
      line: 214,
      message: '// FIXME: retry logic for transient failures',
    },
    {
      severity: 'medium',
      category: 'security',
      file: 'crates/clawft-plugin-oauth2/src/lib.rs',
      line: 45,
      message: 'Token refresh interval not validated against minimum bound',
    },
  ],
};

const MOCK_PEER = {
  name: 'frontend-app',
  files: 184,
  loc: 22100,
  coherence: 72.1,
  warnings: 8,
};

// ── Helpers ────────────────────────────────────────────────────

function severityColor(severity: string): string {
  switch (severity) {
    case 'critical':
      return '#dc2626';
    case 'high':
      return '#ea580c';
    case 'warning':
    case 'medium':
      return '#d97706';
    case 'info':
      return '#2563eb';
    default:
      return '#6b7280';
  }
}

function severityBg(severity: string): string {
  switch (severity) {
    case 'critical':
      return '#fef2f2';
    case 'high':
      return '#fff7ed';
    case 'warning':
    case 'medium':
      return '#fffbeb';
    case 'info':
      return '#eff6ff';
    default:
      return '#f9fafb';
  }
}

// ── Component ──────────────────────────────────────────────────

export default function AssessPage() {
  const [report, setReport] = useState<AssessmentReport | null>(null);
  const [kbStatus, setKbStatus] = useState<string>('idle');
  const [running, setRunning] = useState(false);

  // Try to load real assessment data from the KB API on mount.
  useEffect(() => {
    setKbStatus('loading');
    fetch('/api/cdn/kb/weftos-docs.rvf')
      .then((res) => {
        if (res.ok) {
          setKbStatus('available');
        } else {
          setKbStatus('unavailable');
        }
      })
      .catch(() => {
        setKbStatus('unavailable');
      });
  }, []);

  const handleRunAssessment = useCallback(() => {
    setRunning(true);
    // Simulate assessment run with a short delay
    setTimeout(() => {
      setReport(MOCK_REPORT);
      setRunning(false);
    }, 1200);
  }, []);

  return (
    <div style={{ maxWidth: 960, margin: '0 auto', padding: '2rem 1rem' }}>
      <h1 style={{ fontSize: '1.75rem', fontWeight: 700, marginBottom: '0.25rem' }}>
        Assessment Dashboard
      </h1>
      <p style={{ color: '#6b7280', marginBottom: '1.5rem' }}>
        Continuous assessment for WeftOS projects — SOP 4 deployment readiness.
      </p>

      {/* KB status bar */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: '0.5rem',
          padding: '0.5rem 0.75rem',
          background: '#f9fafb',
          borderRadius: 6,
          marginBottom: '1.5rem',
          fontSize: '0.875rem',
        }}
      >
        <span
          style={{
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: kbStatus === 'available' ? '#22c55e' : kbStatus === 'loading' ? '#facc15' : '#ef4444',
            display: 'inline-block',
          }}
        />
        <span>
          Knowledge Base:{' '}
          {kbStatus === 'available'
            ? 'Connected (weftos-docs.rvf)'
            : kbStatus === 'loading'
              ? 'Checking...'
              : 'Unavailable'}
        </span>
      </div>

      {/* Run button */}
      <button
        onClick={handleRunAssessment}
        disabled={running}
        style={{
          padding: '0.5rem 1.25rem',
          background: running ? '#9ca3af' : '#2563eb',
          color: '#fff',
          border: 'none',
          borderRadius: 6,
          cursor: running ? 'default' : 'pointer',
          fontWeight: 600,
          fontSize: '0.875rem',
          marginBottom: '2rem',
        }}
      >
        {running ? 'Running Assessment...' : 'Run Assessment'}
      </button>

      {report && (
        <>
          {/* ── Project Stats ─────────────────────────────── */}
          <section style={{ marginBottom: '2rem' }}>
            <h2 style={{ fontSize: '1.25rem', fontWeight: 600, marginBottom: '0.75rem' }}>
              Project Stats
            </h2>
            <div
              style={{
                display: 'grid',
                gridTemplateColumns: 'repeat(auto-fill, minmax(160px, 1fr))',
                gap: '0.75rem',
              }}
            >
              {[
                { label: 'Files', value: report.summary.total_files },
                { label: 'Lines of Code', value: report.summary.lines_of_code.toLocaleString() },
                { label: 'Rust', value: report.summary.rust_files },
                { label: 'TypeScript', value: report.summary.typescript_files },
                { label: 'Config', value: report.summary.config_files },
                { label: 'Docs', value: report.summary.doc_files },
                { label: 'Dependencies', value: report.summary.dependency_files },
                { label: 'Coherence', value: `${report.summary.coherence_score.toFixed(1)}%` },
              ].map(({ label, value }) => (
                <div
                  key={label}
                  style={{
                    padding: '0.75rem',
                    background: '#f9fafb',
                    borderRadius: 6,
                    textAlign: 'center',
                  }}
                >
                  <div style={{ fontSize: '1.5rem', fontWeight: 700 }}>{value}</div>
                  <div style={{ fontSize: '0.75rem', color: '#6b7280' }}>{label}</div>
                </div>
              ))}
            </div>
          </section>

          {/* ── Findings ──────────────────────────────────── */}
          <section style={{ marginBottom: '2rem' }}>
            <h2 style={{ fontSize: '1.25rem', fontWeight: 600, marginBottom: '0.75rem' }}>
              Findings ({report.findings.length})
            </h2>
            <div style={{ display: 'flex', flexDirection: 'column', gap: '0.5rem' }}>
              {report.findings.map((f, i) => (
                <div
                  key={i}
                  style={{
                    display: 'flex',
                    alignItems: 'flex-start',
                    gap: '0.75rem',
                    padding: '0.625rem 0.75rem',
                    background: severityBg(f.severity),
                    borderLeft: `3px solid ${severityColor(f.severity)}`,
                    borderRadius: 4,
                    fontSize: '0.8125rem',
                  }}
                >
                  <span
                    style={{
                      fontWeight: 600,
                      color: severityColor(f.severity),
                      textTransform: 'uppercase',
                      fontSize: '0.6875rem',
                      minWidth: 60,
                      flexShrink: 0,
                    }}
                  >
                    {f.severity}
                  </span>
                  <div style={{ flex: 1 }}>
                    <div style={{ fontFamily: 'monospace', fontSize: '0.75rem', color: '#374151' }}>
                      {f.file}
                      {f.line != null ? `:${f.line}` : ''}
                    </div>
                    <div style={{ color: '#4b5563', marginTop: 2 }}>{f.message}</div>
                  </div>
                </div>
              ))}
            </div>
          </section>

          {/* ── Peer Comparison ───────────────────────────── */}
          <section>
            <h2 style={{ fontSize: '1.25rem', fontWeight: 600, marginBottom: '0.75rem' }}>
              Peer Comparison
            </h2>
            <table
              style={{
                width: '100%',
                borderCollapse: 'collapse',
                fontSize: '0.8125rem',
              }}
            >
              <thead>
                <tr style={{ borderBottom: '2px solid #e5e7eb' }}>
                  <th style={{ textAlign: 'left', padding: '0.5rem' }}>Metric</th>
                  <th style={{ textAlign: 'right', padding: '0.5rem' }}>{report.project}</th>
                  <th style={{ textAlign: 'right', padding: '0.5rem' }}>{MOCK_PEER.name}</th>
                </tr>
              </thead>
              <tbody>
                {[
                  { metric: 'Files', local: report.summary.total_files, peer: MOCK_PEER.files },
                  {
                    metric: 'Lines of Code',
                    local: report.summary.lines_of_code.toLocaleString(),
                    peer: MOCK_PEER.loc.toLocaleString(),
                  },
                  {
                    metric: 'Coherence',
                    local: `${report.summary.coherence_score.toFixed(1)}%`,
                    peer: `${MOCK_PEER.coherence.toFixed(1)}%`,
                  },
                  {
                    metric: 'Warnings',
                    local: report.summary.complexity_warnings,
                    peer: MOCK_PEER.warnings,
                  },
                ].map(({ metric, local, peer }) => (
                  <tr key={metric} style={{ borderBottom: '1px solid #f3f4f6' }}>
                    <td style={{ padding: '0.5rem' }}>{metric}</td>
                    <td style={{ textAlign: 'right', padding: '0.5rem', fontVariantNumeric: 'tabular-nums' }}>
                      {local}
                    </td>
                    <td style={{ textAlign: 'right', padding: '0.5rem', fontVariantNumeric: 'tabular-nums' }}>
                      {peer}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </section>

          {/* ── Timestamp ─────────────────────────────────── */}
          <p style={{ marginTop: '2rem', fontSize: '0.75rem', color: '#9ca3af' }}>
            Assessment ran at {new Date(report.timestamp).toLocaleString()} | Scope: {report.scope}
          </p>
        </>
      )}
    </div>
  );
}
