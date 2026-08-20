import {
  getAssetsAppSdkClientWithSession,
  type AssetItem as CatalogAssetItem,
  type MediaResource,
} from '@sdkwork/agents-pc-core/sdk/assetsAppSdkClient';
import { agentsDriveUploadService } from '@sdkwork/agents-pc-core/sdk/driveUploadService';

import type { AssetItem } from '../components/AssetDetailModal';

async function resolveResourceUrl(resource: MediaResource | undefined): Promise<string | undefined> {
  if (!resource) return undefined;
  const directUrl = resource.url ?? resource.publicUrl;
  if (directUrl) return directUrl;
  if (!resource.uri) return undefined;
  if (!resource.uri.startsWith('drive://')) return resource.uri;
  return agentsDriveUploadService.resolvePreviewUrl(resource.uri);
}

async function resolveAssetUrl(asset: CatalogAssetItem): Promise<string | undefined> {
  const snapshotUrl = await resolveResourceUrl(asset.resourceSnapshot);
  if (snapshotUrl) return snapshotUrl;
  if (asset.driveUri) {
    return agentsDriveUploadService.resolvePreviewUrl(asset.driveUri);
  }
  return agentsDriveUploadService.resolvePreviewUrl(
    `drive://spaces/${asset.driveSpaceId}/nodes/${asset.driveNodeId}`,
  );
}

function toAssetType(asset: CatalogAssetItem): AssetItem['type'] | null {
  if (asset.assetKind === 'image') return 'image';
  if (asset.assetKind === 'video') return 'video';
  if (asset.assetKind === 'audio') return 'audio';
  if (asset.assetKind === 'document') return 'document';
  return null;
}

function toAspectRatio(resource: MediaResource | undefined): string {
  if (!resource?.width || !resource.height) return '原始比例';
  return `${resource.width}:${resource.height}`;
}

function toResolution(resource: MediaResource | undefined): string {
  if (!resource?.width || !resource.height) return '原始';
  return `${resource.width} × ${resource.height}`;
}

async function toAssetItem(asset: CatalogAssetItem): Promise<AssetItem | null> {
  const type = toAssetType(asset);
  if (!type) return null;
  const mediaUrl = await resolveAssetUrl(asset);
  if (!mediaUrl) return null;
  const posterUrl = await resolveResourceUrl(asset.resourceSnapshot?.poster);
  const thumbnailUrls = await Promise.all(
    (asset.resourceSnapshot?.thumbnails ?? []).map(resolveResourceUrl),
  );
  const imageUrl = posterUrl ?? thumbnailUrls.find(Boolean) ?? mediaUrl;
  return {
    id: asset.assetId,
    imageUrl,
    mediaUrl,
    type,
    prompt: asset.description || asset.title,
    model: asset.resourceSnapshot?.ai?.model || asset.sourceDomain || 'SDKWork',
    aspectRatio: toAspectRatio(asset.resourceSnapshot),
    resolution: toResolution(asset.resourceSnapshot),
    thumbnails: thumbnailUrls.filter((url): url is string => Boolean(url)),
  };
}

function formatGroupDate(createdAt: string): string {
  const date = new Date(createdAt);
  if (Number.isNaN(date.getTime())) return createdAt;
  return new Intl.DateTimeFormat('zh-CN', {
    month: 'long',
    day: 'numeric',
  }).format(date);
}

export class AssetsService {
  static async getAssetGroups(kind?: AssetItem['type']): Promise<{ date: string; items: AssetItem[] }[]> {
    const page = await getAssetsAppSdkClientWithSession().assets.list({ pageSize: 200, kind });
    const mapped = await Promise.all(page.items.map(async (asset) => ({
      asset,
      item: await toAssetItem(asset),
    })));
    const groups = new Map<string, AssetItem[]>();
    for (const { asset, item } of mapped) {
      if (!item) continue;
      const date = formatGroupDate(asset.createdAt);
      groups.set(date, [...(groups.get(date) ?? []), item]);
    }
    return [...groups.entries()].map(([date, items]) => ({ date, items }));
  }
}
