import assert from 'node:assert/strict';
import test from 'node:test';

import type { SdkworkAgentsDriveAppClient } from '@sdkwork/agents-pc-core/sdk';
import { AgentsDriveUploadService } from '@sdkwork/agents-pc-core/sdk';

function createClient(state: 'completed' | 'uploading' = 'completed'): SdkworkAgentsDriveAppClient {
  return {
    uploader: {
      uploadByProfile: async (profile: string, request: {
        appResourceType: string;
        appResourceId: string;
      }) => {
        assert.equal(profile, 'image');
        assert.equal(request.appResourceType, 'agent-session-image');
        assert.equal(request.appResourceId, 'agent-1:session-1');
        return {
          uploadItem: {
            id: 'upload-1',
            spaceId: 'space-1',
            nodeId: 'node-1',
            originalFileName: 'image.png',
            contentType: 'image/png',
            contentLength: '4',
          },
          uploadSession: { state },
          parts: [],
        };
      },
    },
    drive: {
      nodes: {
        downloadUrls: {
          retrieve: async (nodeId: string) => ({
            downloadUrl: `https://download.example.test/${nodeId}`,
            signedSourceUrl: `https://storage.example.test/${nodeId}`,
            expiresAtEpochMs: String(Date.now() + 900_000),
            method: 'GET',
          }),
        },
      },
    },
  } as unknown as SdkworkAgentsDriveAppClient;
}

test('normalizes completed Drive uploads into canonical media resources', async () => {
  const service = new AgentsDriveUploadService(() => createClient());
  const media = await service.upload({
    file: new File(['test'], 'image.png', { type: 'image/png' }),
    purpose: 'agent-chat-image',
    resourceId: 'agent-1:session-1',
  });

  assert.equal(media.source, 'drive');
  assert.equal(media.uri, 'drive://spaces/space-1/nodes/node-1');
  assert.equal(media.id, 'node-1');
  assert.equal(media.url, 'https://download.example.test/node-1');
  assert.equal(media.metadata.driveNodeId, 'node-1');
});

test('rejects invalid media before calling Drive', async () => {
  const service = new AgentsDriveUploadService(() => createClient());
  await assert.rejects(
    service.upload({
      file: new File(['text'], 'notes.txt', { type: 'text/plain' }),
      purpose: 'agent-avatar',
      resourceId: 'agent-1',
    }),
    /not an image/u,
  );
  await assert.rejects(
    service.upload({
      file: new File(['video'], 'voice.mp4', { type: 'video/mp4' }),
      purpose: 'agent-chat-voice',
      resourceId: 'agent-1:session-1',
    }),
    /not audio/u,
  );
});

test('uses media-specific Drive profiles for creative uploads', async () => {
  const expected = [
    ['agent-creative-image', 'image', 'agent-creative-image', 'image/png'],
    ['agent-creative-audio', 'audio', 'agent-creative-audio', 'audio/mpeg'],
    ['agent-creative-video', 'video', 'agent-creative-video', 'video/mp4'],
  ] as const;

  for (const [purpose, profile, appResourceType, mimeType] of expected) {
    let called = false;
    const client = {
      uploader: {
        uploadByProfile: async (actualProfile: string, request: { appResourceType: string }) => {
          called = true;
          assert.equal(actualProfile, profile);
          assert.equal(request.appResourceType, appResourceType);
          return {
            uploadItem: {
              id: `upload-${profile}`,
              spaceId: 'space-1',
              nodeId: `node-${profile}`,
              originalFileName: `creative.${profile}`,
              contentType: mimeType,
              contentLength: '4',
            },
            uploadSession: { state: 'completed' },
            parts: [],
          };
        },
      },
      drive: {
        nodes: {
          downloadUrls: {
            retrieve: async (nodeId: string) => ({
              downloadUrl: `https://download.example.test/${nodeId}`,
              signedSourceUrl: `https://storage.example.test/${nodeId}`,
              expiresAtEpochMs: String(Date.now() + 900_000),
              method: 'GET',
            }),
          },
        },
      },
    } as unknown as SdkworkAgentsDriveAppClient;
    const service = new AgentsDriveUploadService(() => client);
    const media = await service.upload({
      file: new File(['test'], `creative.${profile}`, { type: mimeType }),
      purpose,
      resourceId: 'creative-session-1',
    });
    assert.equal(called, true);
    assert.equal(media.kind, profile);
  }
});

test('rejects incomplete Drive upload sessions', async () => {
  const service = new AgentsDriveUploadService(() => createClient('uploading'));
  await assert.rejects(
    service.upload({
      file: new File(['test'], 'image.png', { type: 'image/png' }),
      purpose: 'agent-chat-image',
      resourceId: 'agent-1:session-1',
    }),
    /completed resource identity/u,
  );
});

test('resolves previews only from canonical Drive URIs', async () => {
  const service = new AgentsDriveUploadService(() => createClient());
  assert.equal(
    await service.resolvePreviewUrl('drive://spaces/space-1/nodes/node-1'),
    'https://download.example.test/node-1',
  );
  await assert.rejects(service.resolvePreviewUrl('blob:local-preview'), /Invalid canonical Drive URI/u);
});
