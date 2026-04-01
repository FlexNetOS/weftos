/**
 * ConsolePan block — terminal emulator shell.
 *
 * In the full implementation this would use xterm.js with WebGL addon.
 * For K8.1, we provide a lightweight inline terminal that sends commands
 * to the kernel ShellAdapter via Tauri invoke (or echoes locally).
 */

import { useState, useRef, useCallback, useEffect } from 'react';
import { registerBlock } from '../engine';
import type { BlockComponentProps } from '../engine';

interface HistoryEntry {
  input: string;
  output: string;
  isError: boolean;
}

async function executeCommand(cmd: string): Promise<{ output: string; isError: boolean }> {
  if (window.__TAURI_INTERNALS__) {
    try {
      const { invoke } = await import('@tauri-apps/api/core');
      const resp = await invoke<{ ok: boolean; data?: unknown; error?: string }>('kernel_status');
      if (cmd.trim() === 'kernel.status' || cmd.trim() === 'status') {
        return {
          output: resp.ok
            ? JSON.stringify(resp.data, null, 2)
            : `Error: ${resp.error ?? 'unknown'}`,
          isError: !resp.ok,
        };
      }
      return { output: `Command sent: ${cmd}`, isError: false };
    } catch (err) {
      return { output: `Tauri error: ${String(err)}`, isError: true };
    }
  }

  // Mock responses for browser mode
  const trimmed = cmd.trim().toLowerCase();
  if (trimmed === 'help') {
    return {
      output: [
        'Available commands:',
        '  status        - Show kernel status',
        '  processes     - List running processes',
        '  chain         - Show recent chain events',
        '  clear         - Clear terminal',
        '  help          - Show this help',
      ].join('\n'),
      isError: false,
    };
  }
  if (trimmed === 'status' || trimmed === 'kernel.status') {
    return {
      output: [
        'Kernel Status: healthy',
        'Version: 0.1.0',
        'Uptime: 142s',
        'Processes: 5',
        'Chain Height: 1042',
      ].join('\n'),
      isError: false,
    };
  }
  if (trimmed === 'processes') {
    return {
      output: [
        'PID  AGENT        STATE',
        '  1  weaver-0     Running',
        '  2  coder-1      Running',
        '  3  reviewer-2   Running',
        '  4  planner-3    Starting',
        '  5  mesh-4       Suspended',
      ].join('\n'),
      isError: false,
    };
  }
  if (trimmed === 'chain') {
    return {
      output: [
        '#1040 AgentSpawn    2026-03-31T10:00:00Z  a1b2c3',
        '#1041 ConfigSet     2026-03-31T10:00:03Z  d4e5f6',
        '#1042 GovernCheck   2026-03-31T10:00:06Z  789abc',
      ].join('\n'),
      isError: false,
    };
  }
  if (trimmed === '') {
    return { output: '', isError: false };
  }
  return { output: `Unknown command: ${cmd}\nType "help" for available commands.`, isError: true };
}

function TerminalBlock({ resolvedProps }: BlockComponentProps) {
  const initialCommand = resolvedProps.initialCommand as string | undefined;
  const [history, setHistory] = useState<HistoryEntry[]>([]);
  const [input, setInput] = useState('');
  const [cmdHistory, setCmdHistory] = useState<string[]>([]);
  const [historyIdx, setHistoryIdx] = useState(-1);
  const scrollRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = useCallback(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, []);

  const submit = useCallback(async (cmd: string) => {
    if (cmd.trim() === 'clear') {
      setHistory([]);
      return;
    }
    const result = await executeCommand(cmd);
    if (cmd.trim()) {
      setCmdHistory((prev) => [...prev, cmd]);
    }
    setHistoryIdx(-1);
    setHistory((prev) => [...prev, { input: cmd, output: result.output, isError: result.isError }]);
    setTimeout(scrollToBottom, 10);
  }, [scrollToBottom]);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLInputElement>) => {
      if (e.key === 'Enter') {
        submit(input);
        setInput('');
      } else if (e.key === 'ArrowUp') {
        e.preventDefault();
        if (cmdHistory.length > 0) {
          const next = historyIdx < 0 ? cmdHistory.length - 1 : Math.max(0, historyIdx - 1);
          setHistoryIdx(next);
          setInput(cmdHistory[next]);
        }
      } else if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (historyIdx >= 0) {
          const next = historyIdx + 1;
          if (next >= cmdHistory.length) {
            setHistoryIdx(-1);
            setInput('');
          } else {
            setHistoryIdx(next);
            setInput(cmdHistory[next]);
          }
        }
      }
    },
    [input, cmdHistory, historyIdx, submit],
  );

  // Execute initial command on mount
  useEffect(() => {
    if (initialCommand) {
      submit(initialCommand);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div className="border border-gray-700 rounded-lg bg-gray-950 font-mono text-sm flex flex-col min-h-[200px] max-h-[500px]">
      {/* Terminal header */}
      <div className="flex items-center gap-2 px-3 py-1.5 border-b border-gray-800 bg-gray-900/80">
        <div className="flex gap-1.5">
          <span className="w-2.5 h-2.5 rounded-full bg-red-500/60" />
          <span className="w-2.5 h-2.5 rounded-full bg-amber-500/60" />
          <span className="w-2.5 h-2.5 rounded-full bg-emerald-500/60" />
        </div>
        <span className="text-xs text-gray-500">WeftOS Console</span>
      </div>

      {/* Scrollable output */}
      <div ref={scrollRef} className="flex-1 overflow-y-auto p-3 space-y-1">
        {history.map((entry, i) => (
          <div key={i}>
            {entry.input && (
              <div className="flex gap-2">
                <span className="text-indigo-400">$</span>
                <span className="text-gray-200">{entry.input}</span>
              </div>
            )}
            {entry.output && (
              <pre className={`whitespace-pre-wrap ml-4 ${entry.isError ? 'text-red-400' : 'text-gray-400'}`}>
                {entry.output}
              </pre>
            )}
          </div>
        ))}
      </div>

      {/* Input line */}
      <div className="flex items-center gap-2 px-3 py-2 border-t border-gray-800">
        <span className="text-indigo-400">$</span>
        <input
          type="text"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          onKeyDown={handleKeyDown}
          className="flex-1 bg-transparent text-gray-200 outline-none"
          placeholder="Type a command..."
          spellCheck={false}
          autoComplete="off"
        />
      </div>
    </div>
  );
}

registerBlock('ConsolePan', TerminalBlock);
export default TerminalBlock;
