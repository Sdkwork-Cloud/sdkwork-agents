import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import test from 'node:test';

import type {
  FeedItem,
  SdkworkFeedsClient,
} from '@sdkwork/agents-pc-core/sdk/feedsOpenSdkClient';
import {
  configureFeedsOpenSdkClientProvider,
  resetFeedsOpenSdkClient,
} from '@sdkwork/agents-pc-core/sdk/feedsOpenSdkClient';
import {
  configureGenerationsAppSdkClientProvider,
  resetGenerationsAppSdkClient,
  type GenerationRecord,
  type SdkworkGenerationsAppClient,
} from '@sdkwork/agents-pc-core/sdk/generationsAppSdkClient';
import {
  configureSkillsAppSdkClientProvider,
  resetSkillsAppSdkClient,
  type SdkworkSkillsAppClient,
} from '@sdkwork/agents-pc-core/sdk/skillsAppSdkClient';
import {
  configureDriveAppSdkClientProvider,
  resetDriveAppSdkClient,
  type SdkworkAgentsDriveAppClient,
} from '@sdkwork/agents-pc-core/sdk';
import {
  applyAppSdkSessionTokens,
  clearAppSdkSessionTokens,
} from '@sdkwork/agents-pc-core/session';

import { AssetsService } from '../packages/sdkwork-agents-pc-assets/src/services/AssetsService';
import { CanvasService } from '../packages/sdkwork-agents-pc-canvas/src/services/CanvasService';
import { CreativeService } from '../packages/sdkwork-agents-pc-creative/src/services/CreativeService';
import { InspirationService } from '../packages/sdkwork-agents-pc-inspiration/src/services/InspirationService';

function record(id: string, modality: 'image' | 'video'): GenerationRecord {
  return {
    id,
    tenantId: '100001',
    userId: '9001',
    modality,
    operationType: modality === 'video' ? 'text_to_video' : 'text_to_image',
    promptPreview: `${modality} prompt`,
    sourceProvider: 'sdkwork-test-provider',
    status: 'succeeded',
    resultCount: 1,
    createdAt: '2026-07-30T00:00:00Z',
    updatedAt: '2026-07-30T00:00:01Z',
  };
}

function createGenerationsClient(calls: string[]): SdkworkGenerationsAppClient {
  const resultPage = (generationId: string) => ({
    items: [{
      id: `${generationId}-result`,
      generationId,
      resultType: generationId.includes('video') ? 'video' : 'image',
      resourceSnapshot: {
        kind: generationId.includes('video') ? 'video' : 'image',
        url: `https://media.example.test/${generationId}`,
      },
      createdAt: '2026-07-30T00:00:01Z',
    }],
    pageInfo: { mode: 'cursor', hasMore: false },
  });
  return {
    generations: {
      images: {
        textToImage: async () => {
          calls.push('images.textToImage');
          return { generation: record('image-generation', 'image') };
        },
      },
      videos: {
        textToVideo: async () => {
          calls.push('videos.textToVideo');
          return { generation: record('video-generation', 'video') };
        },
      },
      get: async (generationId: string) => record(
        generationId,
        generationId.includes('video') ? 'video' : 'image',
      ),
      list: async () => ({
        items: [record('image-generation', 'image')],
        pageInfo: { mode: 'cursor', hasMore: false },
      }),
      results: {
        list: async (generationId: string) => resultPage(generationId),
      },
    },
  } as unknown as SdkworkGenerationsAppClient;
}

function installSession(): void {
  applyAppSdkSessionTokens({
    accessToken: 'test-access-token',
    authToken: 'test-auth-token',
    context: {
      appId: 'sdkwork-agents',
      tenantId: '100001',
      userId: '9001',
    },
  });
}

test('Creative and Canvas use Generations SDK operations and stable UI message ids', async () => {
  const calls: string[] = [];
  installSession();
  configureGenerationsAppSdkClientProvider(() => createGenerationsClient(calls));
  try {
    const updates: string[] = [];
    const creative = await CreativeService.generateContent('生成图片', 'image', (message) => {
      updates.push(message.id);
    });
    assert.equal(new Set(updates).size, 1);
    assert.equal(creative.imageUrl, 'https://media.example.test/image-generation');

    const imageUrl = await CanvasService.generateImage('画布图片', '16:9', () => undefined);
    const videoUrl = await CanvasService.generateVideo('画布视频', () => undefined);
    assert.equal(imageUrl, 'https://media.example.test/image-generation');
    assert.equal(videoUrl, 'https://media.example.test/video-generation');
    assert.deepEqual(calls, [
      'images.textToImage',
      'images.textToImage',
      'videos.textToVideo',
    ]);
  } finally {
    resetGenerationsAppSdkClient();
    clearAppSdkSessionTokens();
  }
});

test('Assets maps Drive SDK records without local or provider mock media', async () => {
  const client = {
    drive: {
      assets: {
        list: async () => ({
          items: [{
            assetId: 'asset-1',
            assetKind: 'image',
            title: '生成图片',
            description: '来自 Drive 的图片',
            sourceDomain: 'generations',
            driveSpaceId: 'space-1',
            driveNodeId: 'node-1',
            driveUri: 'drive://spaces/space-1/nodes/node-1',
            resourceSnapshot: {
              kind: 'image',
              url: 'https://media.example.test/asset-1',
              width: 1024,
              height: 1024,
            },
            createdAt: '2026-07-30T10:00:00Z',
            updatedAt: '2026-07-30T10:00:00Z',
          }],
          pageInfo: { mode: 'offset', page: 1, pageSize: 200, hasMore: false },
        }),
      },
    },
  } as unknown as SdkworkAgentsDriveAppClient;
  configureDriveAppSdkClientProvider(() => client);
  try {
    const groups = await AssetsService.getAssetGroups();
    assert.equal(groups.length, 1);
    assert.equal(groups[0].items[0].imageUrl, 'https://media.example.test/asset-1');
    assert.equal(groups[0].items[0].type, 'image');
  } finally {
    resetDriveAppSdkClient();
  }
});

test('Inspiration maps feeds streams and Skills marketplace SDK pages', async () => {
  const streamListCalls: Array<{ streamKey?: string; pageSize?: number; q?: string }> = [];
  const feedsClient = {
    feeds: {
      streams: {
        items: {
          list: async (streamKey: string, params: { pageSize?: number; q?: string } = {}) => {
            streamListCalls.push({ streamKey, ...params });
            const common: FeedItem = {
              id: '',
              tenantId: '100001',
              streamId: 'stream-1',
              streamKey,
              sourceType: 'community.entry',
              sourceId: '',
              title: '',
              author: { id: 'official', name: '官方团队', avatarUrl: 'https://media.example.test/avatar.jpg' },
              isPinned: false,
              status: 'active',
              publishedAt: '2026-08-01T00:00:00Z',
              createdAt: '2026-08-01T00:00:00Z',
              updatedAt: '2026-08-01T00:00:00Z',
            };
            if (streamKey === 'agents-inspiration-discover') {
              return {
                items: [{
                  ...common,
                  id: 'discover-1',
                  sourceId: 'discover-1',
                  title: '发现作品',
                  excerpt: '发现简介',
                  coverUrl: 'https://media.example.test/discover.jpg',
                  reactionCount: 12,
                  payload: {
                    src: 'https://media.example.test/discover.jpg',
                    prompt: 'discover prompt',
                    isBanner: true,
                  },
                }],
                pageInfo: { mode: 'cursor', pageSize: 20, hasMore: false },
              };
            }
            if (streamKey === 'agents-inspiration-short-video') {
              return {
                items: [{
                  ...common,
                  id: 'video-1',
                  sourceId: 'video-1',
                  title: '短片作品',
                  excerpt: '短片简介',
                  coverUrl: 'https://media.example.test/video.jpg',
                  reactionCount: 12,
                  payload: {
                    cover: 'https://media.example.test/video.jpg',
                    videoUrl: 'https://media.example.test/video.mp4',
                    duration: '00:30',
                  },
                }],
                pageInfo: { mode: 'cursor', pageSize: 20, hasMore: false },
              };
            }
            return {
              items: [{
                ...common,
                id: 'activity-1',
                sourceId: 'activity-1',
                title: '创作活动',
                excerpt: '活动简介',
                coverUrl: 'https://media.example.test/activity.jpg',
                reactionCount: 12,
                payload: {
                  cover: 'https://media.example.test/activity.jpg',
                  banner: 'https://media.example.test/activity-banner.jpg',
                  status: '征集中',
                  tag: '官方活动',
                  background: '活动背景',
                  timeRange: '2026-07-01 - 2026-08-01',
                  works: [],
                },
              }],
              pageInfo: { mode: 'cursor', pageSize: 20, hasMore: false },
            };
          },
        },
      },
    },
  } as unknown as SdkworkFeedsClient;
  const skillsClient = {
    skills: {
      marketplace: {
        list: async () => ({
          items: [{
            id: 'skill-1',
            name: '分镜技能',
            summary: '生成分镜方案',
            organizationId: '0',
            categories: ['video-production'],
            tags: ['author:SDKWork'],
            installCount: '21',
          }],
          pageInfo: { mode: 'offset', page: 1, pageSize: 100, hasMore: false },
        }),
      },
      skillCategories: {
        list: async () => ({
          items: [{ code: 'video-production', name: '短剧影视' }],
          pageInfo: { mode: 'offset', page: 1, pageSize: 100, hasMore: false },
        }),
      },
    },
  } as unknown as SdkworkSkillsAppClient;
  configureFeedsOpenSdkClientProvider(() => feedsClient);
  configureSkillsAppSdkClientProvider(() => skillsClient);
  try {
    const [discover, videos, activities, skills] = await Promise.all([
      InspirationService.getDiscoverData(),
      InspirationService.getShortVideos(),
      InspirationService.getActivities(),
      InspirationService.getSkills(),
    ]);
    assert.equal(discover.banner.src, 'https://media.example.test/discover.jpg');
    assert.equal(videos[0].videoUrl, 'https://media.example.test/video.mp4');
    assert.equal(activities[0].title, '创作活动');
    assert.equal(skills[0].category, '短剧影视');
    assert.equal(skills[0].items[0].author, 'SDKWork');
    assert.equal(streamListCalls.length, 3);
    assert.ok(streamListCalls.every((call) => call.pageSize === 20));
  } finally {
    resetFeedsOpenSdkClient();
    resetSkillsAppSdkClient();
  }
});

test('active PC remote services contain no raw HTTP, manual auth, or fake media providers', () => {
  const sources = [
    '../packages/sdkwork-agents-pc-assets/src/services/AssetsService.ts',
    '../packages/sdkwork-agents-pc-canvas/src/services/CanvasService.ts',
    '../packages/sdkwork-agents-pc-creative/src/services/CreativeService.ts',
    '../packages/sdkwork-agents-pc-inspiration/src/services/InspirationService.ts',
    '../packages/sdkwork-agents-pc-core/src/sdk/generationsService.ts',
  ].map((path) => readFileSync(new URL(path, import.meta.url), 'utf8')).join('\n');

  assert.doesNotMatch(sources, /\bfetch\s*\(|axios|XMLHttpRequest/);
  assert.doesNotMatch(sources, /Authorization|Access-Token/);
  assert.doesNotMatch(sources, /MOCK_|picsum|unsplash|mixkit/i);
});
