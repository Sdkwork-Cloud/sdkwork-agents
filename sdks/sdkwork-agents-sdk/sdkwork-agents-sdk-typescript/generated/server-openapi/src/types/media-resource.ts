import type { MediaAccess } from './media-access';
import type { MediaChecksum } from './media-checksum';
import type { MediaKind } from './media-kind';
import type { MediaSource } from './media-source';

export interface MediaResource {
  id: string;
  kind: MediaKind;
  source: MediaSource;
  uri: string;
  /** Transient Drive delivery hint; never persisted by Agents. */
  url?: string;
  /** Transient Drive delivery hint; never persisted by Agents. */
  publicUrl?: string;
  objectBlobId?: string | null;
  fileName?: string | null;
  mimeType?: string | null;
  sizeBytes?: string | null;
  checksum?: MediaChecksum | null;
  width?: number | null;
  height?: number | null;
  durationSeconds?: number | null;
  altText?: string | null;
  title?: string | null;
  access?: MediaAccess | null;
  metadata?: Record<string, unknown>;
}
