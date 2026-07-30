export interface ShortVideo {
  id: string;
  title: string;
  author: string;
  avatar: string;
  likes: number;
  duration: string;
  desc: string;
  cover: string;
  videoUrl: string;
}

export interface SkillItem {
  id: string;
  title: string;
  desc: string;
  likes: number;
  author: string;
}

export interface SkillCategory {
  category: string;
  items: SkillItem[];
}

export interface ActivityWork {
  id: string;
  title: string;
  author: string;
  avatar: string;
  likes: number;
  duration: string;
  cover: string;
  videoUrl: string;
  desc: string;
}

export interface Activity {
  id: string;
  title: string;
  desc: string;
  status: string;
  tag: string;
  participants: number;
  cover: string;
  banner: string;
  background: string;
  timeRange: string;
  works: ActivityWork[];
}

export interface DiscoverItem {
  id: string;
  src: string;
  alt: string;
  author: string;
  avatar: string;
  likes: number;
  title?: string;
  prompt: string;
  date?: string;
  aspectRatio?: string;
  model?: string;
  isBanner?: boolean;
}

export interface DiscoverData {
  banner: DiscoverItem;
  cols: DiscoverItem[][];
}
