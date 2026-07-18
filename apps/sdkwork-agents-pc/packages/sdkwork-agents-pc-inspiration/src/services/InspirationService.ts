import { 
  MOCK_SHORT_VIDEOS, 
  MOCK_SKILLS, 
  MOCK_ACTIVITIES,
  DISCOVER_MOCK_DATA,
  FEATURE_CARDS,
  ShortVideo, 
  SkillCategory,
  Activity,
  DiscoverData
} from '../mockData';

export class InspirationService {
  /**
   * Fetch discover tab data
   */
  static async getDiscoverData(): Promise<DiscoverData> {
    await new Promise(resolve => setTimeout(resolve, 300));
    return DISCOVER_MOCK_DATA;
  }

  /**
   * Fetch short videos
   */
  static async getShortVideos(query?: string): Promise<ShortVideo[]> {
    await new Promise(resolve => setTimeout(resolve, 300));
    if (!query) return MOCK_SHORT_VIDEOS;
    const lowerQuery = query.toLowerCase();
    return MOCK_SHORT_VIDEOS.filter(video => 
      video.title.toLowerCase().includes(lowerQuery) ||
      video.author.toLowerCase().includes(lowerQuery) ||
      video.desc.toLowerCase().includes(lowerQuery)
    );
  }

  /**
   * Fetch skills categories
   */
  static async getSkills(query?: string): Promise<SkillCategory[]> {
    await new Promise(resolve => setTimeout(resolve, 300));
    if (!query) return MOCK_SKILLS;
    const lowerQuery = query.toLowerCase();
    return MOCK_SKILLS.map(cat => ({
      category: cat.category,
      items: cat.items.filter(item => 
        item.title.toLowerCase().includes(lowerQuery) ||
        item.desc.toLowerCase().includes(lowerQuery) ||
        item.author.toLowerCase().includes(lowerQuery)
      )
    })).filter(cat => cat.items.length > 0);
  }

  /**
   * Fetch activities
   */
  static async getActivities(query?: string): Promise<Activity[]> {
    await new Promise(resolve => setTimeout(resolve, 300));
    if (!query) return MOCK_ACTIVITIES;
    const lowerQuery = query.toLowerCase();
    return MOCK_ACTIVITIES.filter(act => 
      act.title.toLowerCase().includes(lowerQuery) ||
      act.desc.toLowerCase().includes(lowerQuery)
    );
  }
}
