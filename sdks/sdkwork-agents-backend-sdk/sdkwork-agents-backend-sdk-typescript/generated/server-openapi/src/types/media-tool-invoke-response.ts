import type { DriveAssetView } from './drive-asset-view';

export interface MediaToolInvokeResponse {
  toolCallId: string;
  status: string;
  output: Record<string, unknown>;
  error?: string;
  driveAsset?: DriveAssetView;
}
