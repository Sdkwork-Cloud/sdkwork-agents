import { CreativeSession } from '../types';

export const INITIAL_CREATIVE_SESSIONS: CreativeSession[] = [
  {
    id: 'default',
    title: '默认创作',
    messages: [
      {
        id: 'user-lobster-1',
        role: 'user',
        text: '龙虾的app图标，扁平风格，卡通形象'
      },
      {
        id: 'assistant-lobster-1',
        role: 'assistant',
        text: '龙虾的app图标，扁平风格，卡通形象',
        stage: 'completed',
        modelInfo: '图片 5.0 Lite | 1:1 | 2K',
        imageUrls: [
          'https://images.unsplash.com/photo-1551248429-40975aa4de74?w=500&q=80',
          'https://images.unsplash.com/photo-1607604276583-eef5d076aa5f?w=500&q=80',
          'https://images.unsplash.com/photo-1579783900882-c0d3dad7b119?w=500&q=80',
          'https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=500&q=80'
        ],
        suggestions: [
          "把人物肤色调得更白皙一些",
          "重新生成一版暖黄色调的",
          "用这张图做成社交平台头像"
        ]
      },
      {
        id: 'user-lobster-2',
        role: 'user',
        text: '龙虾的卡通形象，扁平风格，卡通形象，做各种日常动作'
      },
      {
        id: 'assistant-lobster-2',
        role: 'assistant',
        text: '龙虾的卡通形象，扁平风格，卡通形象，做各种日常动作',
        stage: 'completed',
        modelInfo: '图片 5.0 Lite | 1:1 | 2K',
        imageUrls: [
          'https://images.unsplash.com/photo-1634017839464-5c339ebe3cb4?w=500&q=80',
          'https://images.unsplash.com/photo-1620641788421-7a1c342ea42e?w=500&q=80',
          'https://images.unsplash.com/photo-1514888286974-6c03e2ca1dba?w=500&q=80',
          'https://images.unsplash.com/photo-1533738363-b7f9aef128ce?w=500&q=80'
        ],
        suggestions: [
          "把它变成潜水状态",
          "做一版吃面条的龙虾形象",
          "换成3D潮玩盲盒风格"
        ]
      }
    ]
  },
  {
    id: 'recent-beauty',
    title: '生成年轻亚洲美女图片',
    avatarUrl: 'https://images.unsplash.com/photo-1524504388940-b1c1722653e1?w=80&q=80',
    messages: [
      {
        id: 'u-beauty',
        role: 'user',
        text: '生成年轻亚洲美女图片'
      },
      {
        id: 'a-beauty',
        role: 'assistant',
        text: '生成年轻亚洲美女图片',
        stage: 'completed',
        modelInfo: '图片 5.0 Lite | 1:1 | 2K',
        imageUrls: [
          'https://images.unsplash.com/photo-1524504388940-b1c1722653e1?w=500&q=80',
          'https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=500&q=80',
          'https://images.unsplash.com/photo-1517841905240-472988babdf9?w=500&q=80',
          'https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=500&q=80'
        ],
        suggestions: [
          "把人物肤色调得更白皙一些",
          "重新生成一版暖黄色调的"
        ]
      }
    ]
  }
];
