import { AssetItem } from '@/packages/sdkwork-chatbox-pc-assets/src/components/AssetDetailModal';

const lobsterImg = '/src/assets/images/cartoon_lobster_1783657276541.jpg';

// Enriched Mock data generator for interactive assets
const LOBSTER_PROMPT = '龙虾的卡通形象，扁平风格，卡通形象，';

const createItem = (id: string, index: number, dateSeed: string): AssetItem => {
  if (id === 'lobster-01') {
    return {
      id,
      imageUrl: lobsterImg,
      type: 'image',
      prompt: LOBSTER_PROMPT,
      model: '5.0 Lite',
      aspectRatio: '1:1',
      resolution: '2K',
      thumbnails: [
        lobsterImg,
        'https://picsum.photos/seed/lobster_sunglasses/400/400',
        'https://picsum.photos/seed/lobster_cooking/400/400',
        'https://picsum.photos/seed/lobster_beach/400/400'
      ]
    };
  }
  
  const isVideo = index % 8 === 4; // mix some videos
  const aspectRatios = ['1:1', '16:9', '9:16', '4:3'];
  const aspect = aspectRatios[index % aspectRatios.length];
  
  let imageUrl = `https://picsum.photos/seed/${dateSeed}-${index}/600/600`;
  if (aspect === '16:9') imageUrl = `https://picsum.photos/seed/${dateSeed}-${index}/800/450`;
  if (aspect === '9:16') imageUrl = `https://picsum.photos/seed/${dateSeed}-${index}/450/800`;
  if (aspect === '4:3') imageUrl = `https://picsum.photos/seed/${dateSeed}-${index}/800/600`;

  const videoUrls = [
    'https://assets.mixkit.co/videos/preview/mixkit-forest-stream-in-the-sunlight-529-large.mp4',
    'https://assets.mixkit.co/videos/preview/mixkit-waterfall-in-forest-2213-large.mp4',
    'https://assets.mixkit.co/videos/preview/mixkit-waves-in-the-ocean-at-sunset-44331-large.mp4',
    'https://assets.mixkit.co/videos/preview/mixkit-misty-mountains-under-golden-sky-41604-large.mp4'
  ];

  return {
    id,
    imageUrl: isVideo ? `https://picsum.photos/seed/${dateSeed}-video-${index}/800/450` : imageUrl,
    type: isVideo ? 'video' : 'image',
    prompt: `生成的艺术创意图片，包含多种材质融合、光影对比，采用先进的多模态生成模型，种子值为 ${index * 137}。`,
    model: '5.0 Lite',
    aspectRatio: aspect,
    resolution: isVideo ? '1080P' : '2K',
    thumbnails: isVideo ? videoUrls : [
      imageUrl,
      `https://picsum.photos/seed/${dateSeed}-var1-${index}/600/600`,
      `https://picsum.photos/seed/${dateSeed}-var2-${index}/600/600`,
      `https://picsum.photos/seed/${dateSeed}-var3-${index}/600/600`
    ]
  };
};

const MOCK_GROUPS: { date: string; items: AssetItem[] }[] = [
  {
    date: '3月18日',
    items: [
      createItem('lobster-01', 0, 'lobster'),
      ...Array.from({ length: 21 }).map((_, i) => createItem(`0318-${i+1}`, i + 1, 'lobster18'))
    ]
  },
  {
    date: '3月14日',
    items: Array.from({ length: 20 }).map((_, i) => createItem(`0314-${i}`, i, 'lobster14'))
  },
  {
    date: '3月12日',
    items: Array.from({ length: 4 }).map((_, i) => createItem(`0312-${i}`, i, 'lobster12'))
  },
  {
    date: '3月2日',
    items: Array.from({ length: 12 }).map((_, i) => createItem(`0302-${i}`, i, 'lobster02'))
  }
];

export class AssetsService {
  /**
   * Fetch all user assets grouped by date
   */
  static async getAssetGroups(): Promise<{ date: string; items: AssetItem[] }[]> {
    await new Promise(resolve => setTimeout(resolve, 300));
    return MOCK_GROUPS;
  }
}
