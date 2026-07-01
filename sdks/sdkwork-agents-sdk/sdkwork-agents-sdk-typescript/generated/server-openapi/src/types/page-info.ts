import type { Int64String } from './int64-string';

export interface PageInfo {
  /** Pagination mode used by the response. */
  mode: 'offset' | 'cursor';
  /** Current page number when mode is offset. */
  page?: number;
  /** Page size used by the response. */
  pageSize?: number;
  totalItems?: Int64String;
  /** Total pages when mode is offset. */
  totalPages?: number;
  /** Opaque cursor for the next page when mode is cursor. */
  nextCursor?: string | null;
  /** Whether more pages follow the current page. */
  hasMore?: boolean;
}
