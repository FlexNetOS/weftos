/**
 * Session & config persistence for the clawft WASM sandbox.
 *
 * - IndexedDB: large data (conversation history, assessment results)
 * - localStorage: small config (theme, panel visibility, preferences)
 */

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

export interface SessionMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  ts?: number;
}

export interface AssessmentResult {
  id: string;
  repoUrl: string;
  timestamp: number;
  summary: Record<string, unknown>;
  findings: Array<{
    severity: string;
    category: string;
    file: string;
    line?: number;
    message: string;
  }>;
}

export interface UserPreferences {
  theme: 'system' | 'light' | 'dark';
  chainPanelVisible: boolean;
  docPanelVisible: boolean;
  kbGraphExpanded: boolean;
  model: string;
}

// ---------------------------------------------------------------------------
// localStorage helpers (preferences)
// ---------------------------------------------------------------------------

const PREFS_KEY = 'clawft-preferences';

const DEFAULT_PREFS: UserPreferences = {
  theme: 'system',
  chainPanelVisible: true,
  docPanelVisible: true,
  kbGraphExpanded: true,
  model: 'openrouter/google/gemini-2.0-flash-001',
};

export function loadPreferences(): UserPreferences {
  try {
    const raw = localStorage.getItem(PREFS_KEY);
    if (!raw) return { ...DEFAULT_PREFS };
    return { ...DEFAULT_PREFS, ...JSON.parse(raw) };
  } catch {
    return { ...DEFAULT_PREFS };
  }
}

export function savePreferences(prefs: Partial<UserPreferences>): void {
  try {
    const current = loadPreferences();
    const merged = { ...current, ...prefs };
    localStorage.setItem(PREFS_KEY, JSON.stringify(merged));
  } catch {
    // localStorage unavailable — silently ignore
  }
}

// ---------------------------------------------------------------------------
// IndexedDB helpers (conversations, assessments)
// ---------------------------------------------------------------------------

const DB_NAME = 'clawft-session';
const DB_VERSION = 1;
const STORE_CONVERSATIONS = 'conversations';
const STORE_ASSESSMENTS = 'assessments';

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const req = indexedDB.open(DB_NAME, DB_VERSION);

    req.onupgradeneeded = () => {
      const db = req.result;
      if (!db.objectStoreNames.contains(STORE_CONVERSATIONS)) {
        db.createObjectStore(STORE_CONVERSATIONS, { keyPath: 'id' });
      }
      if (!db.objectStoreNames.contains(STORE_ASSESSMENTS)) {
        const store = db.createObjectStore(STORE_ASSESSMENTS, { keyPath: 'id' });
        store.createIndex('by-timestamp', 'timestamp', { unique: false });
      }
    };

    req.onsuccess = () => resolve(req.result);
    req.onerror = () => reject(req.error);
  });
}

/** Generic put into an object store. */
async function idbPut<T>(storeName: string, value: T): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(storeName, 'readwrite');
    tx.objectStore(storeName).put(value);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

/** Generic get from an object store by key. */
async function idbGet<T>(storeName: string, key: string): Promise<T | undefined> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(storeName, 'readonly');
    const req = tx.objectStore(storeName).get(key);
    req.onsuccess = () => resolve(req.result as T | undefined);
    req.onerror = () => reject(req.error);
  });
}

/** Generic getAll from an object store. */
async function idbGetAll<T>(storeName: string): Promise<T[]> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(storeName, 'readonly');
    const req = tx.objectStore(storeName).getAll();
    req.onsuccess = () => resolve(req.result as T[]);
    req.onerror = () => reject(req.error);
  });
}

/** Generic delete from an object store. */
async function idbDelete(storeName: string, key: string): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(storeName, 'readwrite');
    tx.objectStore(storeName).delete(key);
    tx.oncomplete = () => resolve();
    tx.onerror = () => reject(tx.error);
  });
}

// ---------------------------------------------------------------------------
// Conversation persistence
// ---------------------------------------------------------------------------

interface StoredConversation {
  id: string;
  messages: SessionMessage[];
  updatedAt: number;
}

const ACTIVE_CONV_ID = 'active';

export async function saveConversation(messages: SessionMessage[]): Promise<void> {
  const record: StoredConversation = {
    id: ACTIVE_CONV_ID,
    messages,
    updatedAt: Date.now(),
  };
  await idbPut(STORE_CONVERSATIONS, record);
}

export async function loadConversation(): Promise<SessionMessage[]> {
  try {
    const record = await idbGet<StoredConversation>(STORE_CONVERSATIONS, ACTIVE_CONV_ID);
    return record?.messages ?? [];
  } catch {
    return [];
  }
}

export async function clearConversation(): Promise<void> {
  try {
    await idbDelete(STORE_CONVERSATIONS, ACTIVE_CONV_ID);
  } catch {
    // ignore
  }
}

// ---------------------------------------------------------------------------
// Assessment persistence
// ---------------------------------------------------------------------------

export async function saveAssessment(result: AssessmentResult): Promise<void> {
  await idbPut(STORE_ASSESSMENTS, result);
}

export async function loadAssessments(): Promise<AssessmentResult[]> {
  try {
    return await idbGetAll<AssessmentResult>(STORE_ASSESSMENTS);
  } catch {
    return [];
  }
}

export async function loadAssessment(id: string): Promise<AssessmentResult | undefined> {
  try {
    return await idbGet<AssessmentResult>(STORE_ASSESSMENTS, id);
  } catch {
    return undefined;
  }
}

export async function deleteAssessment(id: string): Promise<void> {
  try {
    await idbDelete(STORE_ASSESSMENTS, id);
  } catch {
    // ignore
  }
}
