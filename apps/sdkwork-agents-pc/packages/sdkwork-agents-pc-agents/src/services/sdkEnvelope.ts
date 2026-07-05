type RecordLike = Record<string, unknown>;

export function isRecord(value: unknown): value is RecordLike {
  return Boolean(value) && typeof value === 'object' && !Array.isArray(value);
}

export function extractArray(value: unknown): unknown[] {
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

export function extractResourceRecord(value: unknown): RecordLike {
  if (!isRecord(value)) {
    return {};
  }
  if (isRecord(value.item)) {
    return value.item;
  }
  if (isRecord(value.data)) {
    if (isRecord(value.data.item)) {
      return value.data.item;
    }
    return value.data;
  }
  return value;
}

export {
  DEFAULT_LIST_PAGE_SIZE,
  MAX_LIST_PAGE_SIZE,
  extractOffsetPageInfo,
  extractListItems,
  syncAllOffsetPages,
  type OffsetPageInfo,
  type SyncAllOffsetPagesOptions,
} from '@sdkwork/agents-pc-core/sdk/pagination';
