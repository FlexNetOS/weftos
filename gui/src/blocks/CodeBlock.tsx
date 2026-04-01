/**
 * CodeEditor block — syntax-highlighted code viewer with copy button.
 *
 * Uses a simple pre/code approach with Tailwind styling.
 * For full syntax highlighting, integrate a CodeMirror or Shiki instance.
 */

import { useState, useCallback } from 'react';
import { registerBlock } from '../engine';
import type { BlockComponentProps } from '../engine';

function CodeBlock({ resolvedProps }: BlockComponentProps) {
  const value = (resolvedProps.value as string) ?? '';
  const language = (resolvedProps.language as string) ?? 'text';
  const readOnly = (resolvedProps.readOnly as boolean) ?? true;
  const [copied, setCopied] = useState(false);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(value);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch {
      // Clipboard API may not be available in all contexts
    }
  }, [value]);

  return (
    <div className="relative group rounded-lg border border-gray-700 bg-gray-900 overflow-hidden">
      {/* Header bar */}
      <div className="flex items-center justify-between px-3 py-1.5 border-b border-gray-700 bg-gray-800/60">
        <span className="text-xs text-gray-400 font-mono">{language}</span>
        <button
          onClick={handleCopy}
          className="text-xs text-gray-400 hover:text-gray-200 transition-colors px-2 py-0.5 rounded hover:bg-gray-700"
        >
          {copied ? 'Copied' : 'Copy'}
        </button>
      </div>

      {/* Code area */}
      {readOnly ? (
        <pre className="p-3 overflow-x-auto text-sm leading-relaxed">
          <code className="font-mono text-gray-200 whitespace-pre">{value}</code>
        </pre>
      ) : (
        <textarea
          className="w-full p-3 bg-transparent text-sm font-mono text-gray-200 resize-none outline-none min-h-[120px]"
          defaultValue={value}
          spellCheck={false}
        />
      )}
    </div>
  );
}

registerBlock('CodeEditor', CodeBlock);
export default CodeBlock;
