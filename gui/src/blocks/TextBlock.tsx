/**
 * Markdown block — renders markdown content as styled HTML.
 *
 * Uses a lightweight approach: basic markdown parsing without a heavy
 * dependency. For production, swap in react-markdown or similar.
 */

import { useMemo } from 'react';
import { registerBlock } from '../engine';
import type { BlockComponentProps } from '../engine';

function markdownToHtml(md: string): string {
  let html = md
    // Headings
    .replace(/^#### (.+)$/gm, '<h4 class="text-sm font-semibold text-gray-200 mt-3 mb-1">$1</h4>')
    .replace(/^### (.+)$/gm, '<h3 class="text-base font-semibold text-gray-200 mt-4 mb-1">$1</h3>')
    .replace(/^## (.+)$/gm, '<h2 class="text-lg font-semibold text-gray-100 mt-4 mb-2">$1</h2>')
    .replace(/^# (.+)$/gm, '<h1 class="text-xl font-bold text-gray-100 mt-4 mb-2">$1</h1>')
    // Bold + italic
    .replace(/\*\*\*(.+?)\*\*\*/g, '<strong><em>$1</em></strong>')
    .replace(/\*\*(.+?)\*\*/g, '<strong class="text-gray-100">$1</strong>')
    .replace(/\*(.+?)\*/g, '<em>$1</em>')
    // Inline code
    .replace(/`([^`]+)`/g, '<code class="bg-gray-800 text-indigo-300 px-1 py-0.5 rounded text-sm font-mono">$1</code>')
    // Unordered lists
    .replace(/^[-*] (.+)$/gm, '<li class="ml-4 list-disc text-gray-300">$1</li>')
    // Paragraphs (lines not already converted)
    .replace(/^(?!<[hlu])((?!<li).+)$/gm, '<p class="text-gray-300 mb-2">$1</p>');

  // Wrap consecutive <li> in <ul>
  html = html.replace(/((?:<li[^>]*>.*<\/li>\s*)+)/g, '<ul class="mb-2">$1</ul>');

  return html;
}

function MarkdownBlock({ resolvedProps }: BlockComponentProps) {
  const content = (resolvedProps.content as string) ?? '';

  const html = useMemo(() => markdownToHtml(content), [content]);

  return (
    <div
      className="prose prose-invert max-w-none text-sm leading-relaxed"
      dangerouslySetInnerHTML={{ __html: html }}
    />
  );
}

registerBlock('Markdown', MarkdownBlock);
export default MarkdownBlock;
