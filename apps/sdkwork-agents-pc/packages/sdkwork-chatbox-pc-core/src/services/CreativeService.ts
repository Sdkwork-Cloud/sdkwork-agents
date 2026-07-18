import { CreativeSession, CreativeMessage } from '@/packages/sdkwork-chatbox-pc-creative/src/types';
import { INITIAL_CREATIVE_SESSIONS } from '@/packages/sdkwork-chatbox-pc-creative/src/data/mockSessions';
import { uuid } from '@sdkwork/utils';

export class CreativeService {
  /**
   * Fetch all creative sessions for the user.
   */
  static async getSessions(): Promise<CreativeSession[]> {
    // Mock network latency
    await new Promise(resolve => setTimeout(resolve, 300));
    return [...INITIAL_CREATIVE_SESSIONS];
  }

  /**
   * Generate creative content (image, video, etc.) based on a prompt.
   * Mocks a streaming or long-running generation process.
   */
  static async generateContent(prompt: string, mode: string, onUpdate: (message: CreativeMessage) => void): Promise<CreativeMessage> {
    const messageId = uuid();
    const assistantMessage: CreativeMessage = {
      id: messageId,
      role: 'assistant',
      text: prompt,
      stage: 'thinking',
      progress: 0,
      mode: mode,
      modelInfo: mode === 'agent' ? 'Agent Pro | 智能模式' : (mode === 'video' ? '视频 5.0 | 4K' : '图片 5.0 Ultra | 1:1'),
      imageUrls: []
    };

    // Initial state (thinking)
    onUpdate({ ...assistantMessage });

    // Mock progress steps
    await new Promise(resolve => setTimeout(resolve, 1600));
    onUpdate({ ...assistantMessage, stage: 'loading', progress: 29 });

    await new Promise(resolve => setTimeout(resolve, 1400));
    onUpdate({ ...assistantMessage, stage: 'loading', progress: 67 });

    await new Promise(resolve => setTimeout(resolve, 1500));
    
    // Select images based on prompt
    let imgs = [
      'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=500&q=80',
      'https://images.unsplash.com/photo-1620641788421-7a1c342ea42e?w=500&q=80',
      'https://images.unsplash.com/photo-1618005198143-e5283b519a7f?w=500&q=80',
      'https://images.unsplash.com/photo-1563089145-599997674d42?w=500&q=80'
    ];
    const lower = prompt.toLowerCase();
    
    if (lower.includes('美女') || lower.includes('女人') || lower.includes('girl') || lower.includes('woman') || lower.includes('portrait') || lower.includes('人')) {
      imgs = [
        'https://images.unsplash.com/photo-1524504388940-b1c1722653e1?w=500&q=80',
        'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=500&q=80',
        'https://images.unsplash.com/photo-1517841905240-472988babdf9?w=500&q=80',
        'https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=500&q=80'
      ];
    } else if (lower.includes('猫') || lower.includes('cat') || lower.includes('pet')) {
      imgs = [
        'https://images.unsplash.com/photo-1514888286974-6c03e2ca1dba?w=500&q=80',
        'https://images.unsplash.com/photo-1519052537078-e6302a4968d4?w=500&q=80',
        'https://images.unsplash.com/photo-1543466835-00a7907e9de1?w=500&q=80',
        'https://images.unsplash.com/photo-1533738363-b7f9aef128ce?w=500&q=80'
      ];
    } else if (lower.includes('风景') || lower.includes('山') || lower.includes('landscape') || lower.includes('mountain') || lower.includes('nature') || lower.includes('海')) {
      imgs = [
        'https://images.unsplash.com/photo-1506744038136-46273834b3fb?w=500&q=80',
        'https://images.unsplash.com/photo-1470071459604-3b5ec3a7fe05?w=500&q=80',
        'https://images.unsplash.com/photo-1447752875215-b2761acb3c5d?w=500&q=80',
        'https://images.unsplash.com/photo-1472214222541-d510753a8707?w=500&q=80'
      ];
    }

    // Final result
    const finalMessage: CreativeMessage = {
      ...assistantMessage,
      stage: 'completed',
      progress: 100,
      imageUrl: imgs[0],
      imageUrls: imgs,
      videoUrl: mode === 'video' ? 'https://assets.mixkit.co/videos/preview/mixkit-space-exploration-with-a-retro-futuristic-computer-43180-large.mp4' : undefined,
      videoUrls: mode === 'video' ? [
        'https://assets.mixkit.co/videos/preview/mixkit-space-exploration-with-a-retro-futuristic-computer-43180-large.mp4',
        'https://assets.mixkit.co/videos/preview/mixkit-girl-running-on-the-wet-grass-at-sunrise-44754-large.mp4',
        'https://assets.mixkit.co/videos/preview/mixkit-forest-stream-in-the-sunlight-529-large.mp4',
        'https://assets.mixkit.co/videos/preview/mixkit-pink-neon-running-shoes-in-slow-motion-44583-large.mp4'
      ] : undefined,
      suggestions: [
        "尝试更赛博朋克的色彩",
        "切换为极简主义风格",
        "生成动效视频版本"
      ]
    };
    
    onUpdate(finalMessage);
    return finalMessage;
  }
}
