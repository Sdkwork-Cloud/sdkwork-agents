import { CanvasNode, CanvasGroup, Connection } from '@/packages/sdkwork-chatbox-pc-canvas/src/types';

const INITIAL_NODES: CanvasNode[] = [
  {
    id: 'node-1',
    type: 'text',
    x: 140,
    y: 160,
    width: 320,
    height: 250,
    title: '第1步：剧本构想',
    content: '在一个充满赛博朋克霓虹的未来都市中，一个小型的发光Octo章鱼侦探，正在雨夜中寻找神秘的代码卷轴...',
    groupId: 'group-initial'
  },
  {
    id: 'node-2',
    type: 'image-gen',
    x: 550,
    y: 120,
    width: 320,
    height: 380,
    title: '第2步：角色渲染',
    prompt: '在一个充满赛博朋克霓虹的未来都市中，一个小型的发光Octo章鱼侦探，正在雨夜中寻找神秘的代码卷轴...',
    model: '5.0-lite',
    ratio: '16:9',
    status: 'completed',
    mediaUrl: 'https://picsum.photos/seed/octodetective/800/450',
    groupId: 'group-initial'
  },
  {
    id: 'node-3',
    type: 'video-gen',
    x: 1000,
    y: 180,
    width: 320,
    height: 340,
    title: '第3步：动态镜头',
    model: '2.0-fast',
    duration: 5,
    status: 'idle',
    refImageNodeId: 'node-2'
  }
];

const INITIAL_GROUPS: CanvasGroup[] = [
  {
    id: 'group-initial',
    title: '剧本与创意大纲阶段',
    color: 'cyan',
    x: 100,
    y: 60,
    width: 820,
    height: 480,
    nodeIds: ['node-1', 'node-2']
  }
];

const INITIAL_CONNECTIONS: Connection[] = [
  { id: 'conn-1', fromNodeId: 'node-1', toNodeId: 'node-2' },
  { id: 'conn-2', fromNodeId: 'node-2', toNodeId: 'node-3' }
];

const LOCAL_STORAGE_KEY = 'chatbox_canvas_workflow_v1';

export class CanvasService {
  /**
   * Fetch initial workflow state
   */
  static async getInitialWorkflow() {
    await new Promise(resolve => setTimeout(resolve, 300));
    try {
      const saved = localStorage.getItem(LOCAL_STORAGE_KEY);
      if (saved) {
        const parsed = JSON.parse(saved);
        if (parsed && Array.isArray(parsed.nodes)) {
          parsed.nodes = parsed.nodes.map((node: any) => {
            if (node.type === 'image-gen' || node.type === 'video-gen') {
              let updated = false;
              let w = node.width;
              let h = node.height;
              // Reset/Clamp huge dimensions caused by the settings conflict bug
              if (typeof node.width === 'number' && node.width > 360) {
                w = 260;
                updated = true;
              }
              // If it was corrupt/huge, recalculate height using the ratio
              if (updated || (typeof node.height === 'number' && node.height > 550)) {
                let numericRatio = 1.0;
                const ratioStr = node.ratio || '1:1';
                const parts = ratioStr.split(':');
                if (parts.length === 2) {
                  const rW = parseFloat(parts[0]);
                  const rH = parseFloat(parts[1]);
                  if (rW > 0 && rH > 0) {
                    numericRatio = rW / rH;
                  }
                }
                h = Math.round(w / numericRatio) + 37;
                updated = true;
              }
              if (updated) {
                return { ...node, width: w, height: h };
              }
            }
            return node;
          });
        }
        return parsed;
      }
    } catch (e) {
      console.warn('Failed to parse saved workflow', e);
    }
    return {
      nodes: [...INITIAL_NODES],
      groups: [...INITIAL_GROUPS],
      connections: [...INITIAL_CONNECTIONS]
    };
  }

  static saveWorkflow(data: { nodes: CanvasNode[], groups: CanvasGroup[], connections: Connection[] }) {
    try {
      localStorage.setItem(LOCAL_STORAGE_KEY, JSON.stringify(data));
    } catch (e) {
      console.warn('Failed to save workflow', e);
    }
  }

  /**
   * Mock Image Generation
   */
  static async generateImage(prompt: string, ratio: string, onProgress: (p: number, msg: string) => void): Promise<string> {
    const steps = [
      { p: 15, msg: '精简自然语言...' },
      { p: 35, msg: '解耦潜在空间 (Latent Space)...' },
      { p: 60, msg: '像素着色与融合...' },
      { p: 85, msg: '画质超分 polish...' },
      { p: 100, msg: '生成完毕' }
    ];

    for (const step of steps) {
      await new Promise(resolve => setTimeout(resolve, 900));
      onProgress(step.p, step.msg);
    }
    
    const seed = Math.floor(Math.random() * 1000);
    const randAspect = ratio === '16:9' ? '800/450' : ratio === '9:16' ? '450/800' : '600/600';
    return `https://picsum.photos/seed/canvas-${seed}/${randAspect}`;
  }

  /**
   * Mock Video Generation
   */
  static async generateVideo(prompt: string, onProgress: (p: number, msg: string) => void): Promise<string> {
    const steps = [
      { p: 15, msg: '分析文本语义与空间结构...' },
      { p: 35, msg: '生成初始帧向量特征...' },
      { p: 60, msg: '渲染时序连贯性 (Temporal Consistency)...' },
      { p: 85, msg: '增加动态模糊与光影细节...' },
      { p: 100, msg: '生成完毕' }
    ];

    for (const step of steps) {
      await new Promise(resolve => setTimeout(resolve, 900));
      onProgress(step.p, step.msg);
    }
    
    return 'https://assets.mixkit.co/videos/preview/mixkit-waterfall-in-forest-2213-large.mp4';
  }
}
