import type {
  CommunityEntry,
  SdkworkCommunityAppClient,
} from '@sdkwork/agents-pc-core/sdk/communityAppSdkClient';
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

const DISCOVER_TAG = 'agents-inspiration-discover';
const SHORT_VIDEO_TAG = 'agents-inspiration-short-video';
const ACTIVITY_TAG = 'agents-inspiration-activity';
const SKILLS_PAGE_SIZE = 100;
const DISCOVER_COLUMN_COUNT = 6;

async function loadCommunityAppSdkClient(): Promise<SdkworkCommunityAppClient> {
  const { getCommunityAppSdkClientWithSession } = await import(
    '@sdkwork/agents-pc-core/sdk/communityAppSdkClient'
  );
  return getCommunityAppSdkClientWithSession();
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

function parseBody(entry: CommunityEntry): Record<string, unknown> {
  if (!entry.body) return {};
  try {
    const body = JSON.parse(entry.body) as unknown;
    return isRecord(body) ? body : {};
  } catch {
    return {};
  }
}

function toCommunityEntries(items: Record<string, unknown>[]): CommunityEntry[] {
  return items.filter((item): item is Record<string, unknown> & CommunityEntry => (
    typeof item.id === 'string'
    && typeof item.title === 'string'
    && typeof item.body === 'string'
    && isRecord(item.author)
    && isRecord(item.stats)
  ));
}

async function listCommunityEntries(tag: string, query?: string): Promise<CommunityEntry[]> {
  try {
    const client = await loadCommunityAppSdkClient();
    const page = await client.community.feed.list({
      tag,
      reviewState: 'approved',
      page: 1,
      pageSize: 20,
      ...(query?.trim() ? { q: query.trim() } : {}),
    });
    return toCommunityEntries(page.items);
  } catch (error) {
    // The community app service may be unavailable in hosted portal
    // environments; inspiration tabs degrade to empty content.
    console.error(`Failed to load community entries for ${tag}.`, error);
    return [];
  }
}

function toDiscoverItem(entry: CommunityEntry): DiscoverItem | null {
  const body = parseBody(entry);
  const src = readString(body, 'src');
  if (!src) return null;
  return {
    id: entry.id,
    src,
    alt: readString(body, 'alt') || entry.title,
    author: entry.author.name,
    avatar: entry.author.avatarUrl || readString(body, 'avatar'),
    likes: entry.stats.reactionCount ?? 0,
    title: entry.title,
    prompt: readString(body, 'prompt') || entry.excerpt || entry.title,
    date: readString(body, 'date') || entry.publishedAt,
    aspectRatio: readString(body, 'aspectRatio') || undefined,
    model: readString(body, 'model') || undefined,
    isBanner: body.isBanner === true || entry.isFeatured === true,
  };
}

function toShortVideo(entry: CommunityEntry): ShortVideo | null {
  const body = parseBody(entry);
  const cover = readString(body, 'cover');
  const videoUrl = readString(body, 'videoUrl');
  if (!cover || !videoUrl) return null;
  return {
    id: entry.id,
    title: entry.title,
    author: entry.author.name,
    avatar: entry.author.avatarUrl || readString(body, 'avatar'),
    likes: entry.stats.reactionCount ?? 0,
    duration: readString(body, 'duration'),
    desc: entry.excerpt || readString(body, 'desc'),
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

function toActivity(entry: CommunityEntry): Activity | null {
  const body = parseBody(entry);
  const cover = readString(body, 'cover');
  const banner = readString(body, 'banner');
  if (!cover || !banner) return null;
  const works = Array.isArray(body.works)
    ? body.works.map(toActivityWork).filter((work): work is ActivityWork => work !== null)
    : [];
  return {
    id: entry.id,
    title: entry.title,
    desc: entry.excerpt || readString(body, 'desc'),
    status: readString(body, 'status'),
    tag: readString(body, 'tag'),
    participants: readNumber(body, 'participants', entry.stats.viewCount ?? 0),
    cover,
    banner,
    background: readString(body, 'background'),
    timeRange: readString(body, 'timeRange'),
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
    const entries = await listCommunityEntries(DISCOVER_TAG);
    const items = entries.map(toDiscoverItem).filter((item): item is DiscoverItem => item !== null);
    const banner = items.find(item => item.isBanner) ?? items[0];
    if (!banner) {
      throw new Error('发现页没有可展示的 Community 数据。');
    }
    const cols = Array.from({ length: DISCOVER_COLUMN_COUNT }, () => [] as DiscoverItem[]);
    items.filter(item => item.id !== banner.id).forEach((item, index) => {
      cols[index % DISCOVER_COLUMN_COUNT].push(item);
    });
    return { banner, cols };
  }

  /**
   * Fetch short videos
   */
  static async getShortVideos(query?: string): Promise<ShortVideo[]> {
    const entries = await listCommunityEntries(SHORT_VIDEO_TAG, query);
    return entries.map(toShortVideo).filter((video): video is ShortVideo => video !== null);
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
    const entries = await listCommunityEntries(ACTIVITY_TAG, query);
    return entries.map(toActivity).filter((activity): activity is Activity => activity !== null);
  }
}
