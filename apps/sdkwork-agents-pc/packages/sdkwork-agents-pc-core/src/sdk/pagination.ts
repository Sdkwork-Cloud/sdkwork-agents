type RecordLike = Record<string, unknown>;

function isRecord(value: unknown): value is RecordLike {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

function extractArray(value: unknown): unknown[] {
  if (Array.isArray(value)) {
    return value;
  }
  if (!isRecord(value)) {
    return [];
  }
  for (const key of ['items', 'data', 'agents', 'records', 'list']) {
    const nested = value[key];
    if (Array.isArray(nested)) {
      return nested;
    }
    if (isRecord(nested)) {
      const fromNested = extractArray(nested);
      if (fromNested.length > 0) {
        return fromNested;
      }
    }
  }
  return [];
}

function readNumber(record: RecordLike, ...keys: string[]): number {
  for (const key of keys) {
    const value = record[key];
    if (typeof value === 'number' && Number.isFinite(value)) {
      return value;
    }
    if (typeof value === 'string' && value.trim()) {
      const parsed = Number(value);
      if (Number.isFinite(parsed)) {
        return parsed;
      }
    }
  }
  return 0;
}

export const DEFAULT_LIST_PAGE_SIZE = 20;
export const MAX_LIST_PAGE_SIZE = 200;

export interface OffsetPageInfo {
  page: number;
  pageSize: number;
  totalPages: number;
  totalItems: number;
  hasMore: boolean;
}

/** Parse SdkWork offset-mode `pageInfo` from an SDK list response envelope. */
export function extractOffsetPageInfo(value: unknown): OffsetPageInfo {
  const root = isRecord(value) ? value : {};
  const data = isRecord(root.data) ? root.data : root;
  const pageInfo = isRecord(data.pageInfo)
    ? data.pageInfo
    : isRecord(root.pageInfo)
      ? root.pageInfo
      : {};
  const page = Math.max(1, readNumber(pageInfo, 'page'));
  const pageSize = Math.max(1, readNumber(pageInfo, 'pageSize', 'page_size') || DEFAULT_LIST_PAGE_SIZE);
  const totalPages = Math.max(0, readNumber(pageInfo, 'totalPages', 'total_pages'));
  const totalItems = readNumber(pageInfo, 'totalItems', 'total_items');
  const hasMore = pageInfo.hasMore === true || (totalPages > 0 && page < totalPages);
  return { page, pageSize, totalPages, totalItems, hasMore };
}

export interface SyncAllOffsetPagesOptions<TQuery = Record<string, unknown>> {
  pageSize?: number;
  maxPages?: number;
  query?: TQuery;
  mapItems: (response: unknown) => unknown[];
}

/**
 * Export/batch-only helper: follow server `pageInfo.hasMore` across pages.
 * Per `PAGINATION_SPEC.md` §7–§8 — must not back interactive UI tables or feeds.
 */
export async function syncAllOffsetPages<T, TQuery extends Record<string, unknown> = Record<string, unknown>>(
  fetchPage: (params: { page: number; pageSize: number } & TQuery) => Promise<unknown>,
  options: SyncAllOffsetPagesOptions<TQuery>,
): Promise<T[]> {
  const pageSize = Math.min(
    MAX_LIST_PAGE_SIZE,
    Math.max(1, options.pageSize ?? DEFAULT_LIST_PAGE_SIZE),
  );
  const maxPages = options.maxPages ?? 100;
  const query = (options.query ?? {}) as TQuery;
  const items: T[] = [];
  let page = 1;
  while (page <= maxPages) {
    const response = await fetchPage({ page, pageSize, ...query });
    items.push(...(options.mapItems(response) as T[]));
    if (!extractOffsetPageInfo(response).hasMore) {
      break;
    }
    page += 1;
  }
  return items;
}

export function extractListItems(value: unknown): unknown[] {
  return extractArray(value);
}
