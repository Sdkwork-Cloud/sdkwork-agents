import { getDriveAppSdkClientWithSession } from "@sdkwork/agents-pc-core/sdk/driveAppSdkClient";
import { CHAT_FILE_LIBRARY_PROPERTY_KEY } from "@sdkwork/agents-pc-core/sdk/driveUploadService";

/** A chat file library entry backed by a Drive node marked with the library property. */
export interface ChatLibraryFile {
  id: string;
  name: string;
  mimeType?: string;
  sizeBytes?: string;
  updatedAt: string;
  spaceId: string;
}

export interface ChatLibraryPage {
  items: ChatLibraryFile[];
  nextCursor?: string | null;
}

function isNotFoundError(error: unknown): boolean {
  if (!error || typeof error !== 'object') return false;
  const record = error as Record<string, unknown>;
  const status = record.status ?? record.statusCode ?? record.httpStatus;
  if (status === 404 || status === '404') return true;
  if (record.code === 'NOT_FOUND' || record.code === 40401) return true;
  const problem = record.problem;
  if (problem && typeof problem === 'object') {
    const detail = problem as Record<string, unknown>;
    if (detail.status === 404 || detail.status === '404' || detail.code === 40401) return true;
  }
  return false;
}

export class ChatFileLibraryService {
  constructor(
    private readonly getClient: () => ReturnType<typeof getDriveAppSdkClientWithSession> = getDriveAppSdkClientWithSession,
  ) {}

  async listFiles(pageSize = 100, cursor?: string): Promise<ChatLibraryPage> {
    try {
      const result = await this.getClient().drive.propertyNodes.list(CHAT_FILE_LIBRARY_PROPERTY_KEY, {
        pageSize: String(pageSize),
        cursor,
      });
      return {
        items: result.items.map((node) => ({
          id: node.id,
          name: node.nodeName,
          mimeType: node.contentType,
          sizeBytes: node.contentLength,
          updatedAt: node.updatedAt,
          spaceId: node.spaceId,
        })),
        nextCursor: result.pageInfo.nextCursor,
      };
    } catch (error) {
      // The library property may not be provisioned in the caller's Drive yet;
      // an empty library must render as empty instead of failing the view.
      if (isNotFoundError(error)) {
        return { items: [], nextCursor: null };
      }
      throw error;
    }
  }

  async resolvePreviewUrl(nodeId: string): Promise<string> {
    const response = await this.getClient().drive.nodes.downloadUrls.retrieve(nodeId, {
      requestedTtlSeconds: 900,
    });
    return response.downloadUrl;
  }
}

export const chatFileLibraryService = new ChatFileLibraryService();
