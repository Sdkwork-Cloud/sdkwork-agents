import type {
  FeedItem,
  SdkworkFeedsClient,
} from '@sdkwork/agents-pc-core/sdk/feedsOpenSdkClient';
import type {
  SdkworkSkillsAppClient,
  SkillRecord,
} from '@sdkwork/agents-pc-core/sdk/skillsAppSdkClient';

import type {
  Activity,
  ActivityWork,
  DiscoverData,
  DiscoverItem,
  ShortVideo,
  SkillCategory,
} from '../types';

const DISCOVER_STREAM = 'agents-inspiration-discover';
const SHORT_VIDEO_STREAM = 'agents-inspiration-short-video';
const ACTIVITY_STREAM = 'agents-inspiration-activity';
const SKILLS_PAGE_SIZE = 100;
const DISCOVER_COLUMN_COUNT = 6;

async function loadFeedsOpenSdkClient(): Promise<SdkworkFeedsClient> {
  const { getFeedsOpenSdkClient } = await import(
    '@sdkwork/agents-pc-core/sdk/feedsOpenSdkClient'
  );
  return getFeedsOpenSdkClient();
}

async function loadSkillsAppSdkClient(): Promise<SdkworkSkillsAppClient> {
  const { getSkillsAppSdkClient } = await import(
    '@sdkwork/agents-pc-core/sdk/skillsAppSdkClient'
  );
  return getSkillsAppSdkClient();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return Boolean(value && typeof value === 'object' && !Array.isArray(value));
}

function readString(record: Record<string, unknown>, key: string): string {
  const value = record[key];
  return typeof value === 'string' ? value : '';
}

function readNumber(record: Record<string, unknown>, key: string, fallback = 0): number {
  const value = record[key];
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (typeof value === 'string') {
    const parsed = Number(value);
    if (Number.isFinite(parsed)) return parsed;
  }
  return fallback;
}

/**
 * Standardized source payload mapped by the feeds source adapter (whitelisted
 * fields only). Frontends read `payload` instead of parsing raw source bodies.
 */
function payloadOf(item: FeedItem): Record<string, unknown> {
  return isRecord(item.payload) ? item.payload : {};
}

async function listStreamItems(streamKey: string, query?: string): Promise<FeedItem[]> {
  try {
    const client = await loadFeedsOpenSdkClient();
    const page = await client.feeds.streams.items.list(streamKey, {
      pageSize: 20,
      ...(query?.trim() ? { q: query.trim() } : {}),
    });
    return page.items as unknown as FeedItem[];
  } catch (error) {
    // The feeds stream service may be unavailable in hosted portal
    // environments; inspiration tabs degrade to empty content.
    console.error(`Failed to load feed stream ${streamKey}.`, error);
    return [];
  }
}

function toDiscoverItem(item: FeedItem): DiscoverItem | null {
  const payload = payloadOf(item);
  const src = readString(payload, 'src') || item.coverUrl || '';
  if (!src) return null;
  return {
    id: item.id,
    src,
    alt: readString(payload, 'alt') || item.title,
    author: item.author?.name || '',
    avatar: item.author?.avatarUrl || readString(payload, 'avatar'),
    likes: item.reactionCount ?? 0,
    title: item.title,
    prompt: readString(payload, 'prompt') || item.excerpt || item.title,
    date: readString(payload, 'date') || item.publishedAt,
    aspectRatio: readString(payload, 'aspectRatio') || undefined,
    model: readString(payload, 'model') || undefined,
    isBanner: payload.isBanner === true,
  };
}

function toShortVideo(item: FeedItem): ShortVideo | null {
  const payload = payloadOf(item);
  const cover = readString(payload, 'cover') || item.coverUrl || '';
  const videoUrl = readString(payload, 'videoUrl');
  if (!cover || !videoUrl) return null;
  return {
    id: item.id,
    title: item.title,
    author: item.author?.name || '',
    avatar: item.author?.avatarUrl || readString(payload, 'avatar'),
    likes: item.reactionCount ?? 0,
    duration: readString(payload, 'duration'),
    desc: item.excerpt || readString(payload, 'desc'),
    cover,
    videoUrl,
  };
}

function toActivityWork(value: unknown): ActivityWork | null {
  if (!isRecord(value)) return null;
  const id = readString(value, 'id');
  const cover = readString(value, 'cover');
  const videoUrl = readString(value, 'videoUrl');
  if (!id || !cover || !videoUrl) return null;
  return {
    id,
    title: readString(value, 'title'),
    author: readString(value, 'author'),
    avatar: readString(value, 'avatar'),
    likes: readNumber(value, 'likes'),
    duration: readString(value, 'duration'),
    cover,
    videoUrl,
    desc: readString(value, 'desc'),
  };
}

function toActivity(item: FeedItem): Activity | null {
  const payload = payloadOf(item);
  const cover = readString(payload, 'cover') || item.coverUrl || '';
  const banner = readString(payload, 'banner');
  if (!cover || !banner) return null;
  const works = Array.isArray(payload.works)
    ? payload.works.map(toActivityWork).filter((work): work is ActivityWork => work !== null)
    : [];
  return {
    id: item.id,
    title: item.title,
    desc: item.excerpt || readString(payload, 'desc'),
    status: readString(payload, 'status'),
    tag: readString(payload, 'tag'),
    participants: readNumber(payload, 'participants', item.reactionCount ?? 0),
    cover,
    banner,
    background: readString(payload, 'background'),
    timeRange: readString(payload, 'timeRange'),
    works,
  };
}

async function listSkillsPage(query?: string): Promise<SkillRecord[]> {
  try {
    const client = await loadSkillsAppSdkClient();
    const page = await client.skills.marketplace.list({
      pageSize: SKILLS_PAGE_SIZE,
      ...(query?.trim() ? { q: query.trim() } : {}),
    });
    return page.items;
  } catch (error) {
    // The skills app service may be unavailable in hosted portal
    // environments; the skills tab degrades to an empty catalog.
    console.error('Failed to load skills marketplace.', error);
    return [];
  }
}

async function readSkillCategoryLabels(): Promise<Map<string, string>> {
  try {
    const client = await loadSkillsAppSdkClient();
    const page = await client.skills.skillCategories.list({
      pageSize: SKILLS_PAGE_SIZE,
    });
    const labels = new Map<string, string>();
    for (const category of page.items) {
      labels.set(category.code, category.name);
    }
    return labels;
  } catch (error) {
    console.error('Failed to load skill category labels.', error);
    return new Map();
  }
}

function readSkillAuthor(skill: SkillRecord): string {
  return skill.tags.find(tag => tag.startsWith('author:'))?.slice('author:'.length)
    || skill.organizationId;
}

export class InspirationService {
  /**
   * Fetch discover tab data
   */
  static async getDiscoverData(): Promise<DiscoverData> {
    const items = await listStreamItems(DISCOVER_STREAM);
    const discovered = items.map(toDiscoverItem).filter((item): item is DiscoverItem => item !== null);
    const banner = discovered.find(item => item.isBanner) ?? discovered[0];
    if (!banner) {
      throw new Error('发现页没有可展示的灵感数据。');
    }
    const cols = Array.from({ length: DISCOVER_COLUMN_COUNT }, () => [] as DiscoverItem[]);
    discovered.filter(item => item.id !== banner.id).forEach((item, index) => {
      cols[index % DISCOVER_COLUMN_COUNT].push(item);
    });
    return { banner, cols };
  }

  /**
   * Fetch short videos
   */
  static async getShortVideos(query?: string): Promise<ShortVideo[]> {
    const items = await listStreamItems(SHORT_VIDEO_STREAM, query);
    return items.map(toShortVideo).filter((video): video is ShortVideo => video !== null);
  }

  /**
   * Fetch skills categories
   */
  static async getSkills(query?: string): Promise<SkillCategory[]> {
    const [skills, categoryLabels] = await Promise.all([
      listSkillsPage(query),
      readSkillCategoryLabels(),
    ]);
    const groups = new Map<string, SkillCategory['items']>();
    for (const skill of skills) {
      const categoryCodes = skill.categories.length > 0 ? skill.categories : ['other'];
      for (const categoryCode of categoryCodes) {
        const category = categoryLabels.get(categoryCode) || categoryCode;
        groups.set(category, [
          ...(groups.get(category) ?? []),
          {
            id: skill.id,
            title: skill.name,
            desc: skill.summary || skill.description || '',
            likes: Number.parseInt(skill.installCount, 10) || 0,
            author: readSkillAuthor(skill),
          },
        ]);
      }
    }
    return [...groups.entries()].map(([category, items]) => ({ category, items }));
  }

  /**
   * Fetch activities
   */
  static async getActivities(query?: string): Promise<Activity[]> {
    const items = await listStreamItems(ACTIVITY_STREAM, query);
    return items.map(toActivity).filter((activity): activity is Activity => activity !== null);
  }
}
