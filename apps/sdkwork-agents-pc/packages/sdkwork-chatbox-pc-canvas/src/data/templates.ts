import { CanvasNode, CanvasGroup, Connection } from '../types';

export interface Template {
  id: string;
  name: string;
  description: string;
  isSystem?: boolean;
  category: 'workflow' | 'collaboration' | 'brainstorm' | 'custom';
  nodes: CanvasNode[];
  groups: CanvasGroup[];
  connections: Connection[];
}

// System built-in templates to jumpstart workflow creation
export const SYSTEM_TEMPLATES: Template[] = [
  {
    id: 'sys-sequential-chain',
    name: '串联生成工作流 (Sequential Chain)',
    description: '最经典的 AI 创作流。从「灵感/提示词」生成，到「AI 图像高阶渲染」，再到「AI 视频动态演绎」的逐步演进。',
    isSystem: true,
    category: 'workflow',
    nodes: [
      {
        id: 'seq-node-1',
        type: 'text',
        x: 100,
        y: 150,
        width: 280,
        height: 250,
        title: '1. 创意大纲与提示词',
        content: '使用大语言模型生成高质、生动的视觉绘画提示词，以此作为后续图像生成的精准输入。\n\n【核心提示词建议】：\n"A cinematic shot of a futuristic cyberpunk laboratory with neon lights, holographic displays, highly detailed, 8k resolution"'
      },
      {
        id: 'seq-node-2',
        type: 'image-gen',
        x: 460,
        y: 150,
        width: 260,
        height: 300,
        title: '2. AI 图像高阶渲染',
        prompt: 'A cinematic shot of a futuristic cyberpunk laboratory with neon lights, holographic displays, highly detailed, 8k resolution',
        model: 'Gemini 2.5 Flash',
        ratio: '16:9',
        status: 'idle'
      },
      {
        id: 'seq-node-3',
        type: 'video-gen',
        x: 820,
        y: 150,
        width: 260,
        height: 190,
        title: '3. AI 视频动态演绎',
        prompt: 'The holographic displays flicking, subtle steam rises from tubes, slow camera zoom-in',
        model: 'Veo 2.0',
        ratio: '16:9',
        status: 'idle'
      }
    ],
    groups: [
      {
        id: 'seq-group-1',
        title: '线性生成管道 (Creative Pipeline)',
        color: 'cyan',
        x: 50,
        y: 60,
        width: 1100,
        height: 520,
        nodeIds: ['seq-node-1', 'seq-node-2', 'seq-node-3']
      }
    ],
    connections: [
      { id: 'seq-conn-1', fromNodeId: 'seq-node-1', toNodeId: 'seq-node-2' },
      { id: 'seq-conn-2', fromNodeId: 'seq-node-2', toNodeId: 'seq-node-3' }
    ]
  },
  {
    id: 'sys-agent-swarm',
    name: '智能体并联协作 (Agent Swarm)',
    description: '主控节点分发任务，多角色（文案、设计、剪辑）并联展开专业化输出，适合复杂多媒体内容策划。',
    isSystem: true,
    category: 'collaboration',
    nodes: [
      {
        id: 'swarm-node-1',
        type: 'text',
        x: 100,
        y: 280,
        width: 280,
        height: 220,
        title: '🎯 主控分发节点 (Orchestrator)',
        content: '接收原始项目需求：\n「设计一个前沿 AI 智能家居的广告视频脚本与视觉元素」\n\n主控分析任务并同时分发至三个专业智能体子节点。'
      },
      {
        id: 'swarm-node-2',
        type: 'text',
        x: 480,
        y: 50,
        width: 280,
        height: 180,
        title: '✍️ 文案策划智能体 (Copywriter)',
        content: '【分工】撰写广告独白与文字脚本\n【完成度】100%\n【输出】"未来的温度，不需要触碰。每一次呼吸，家都在聆听。"'
      },
      {
        id: 'swarm-node-3',
        type: 'image-gen',
        x: 480,
        y: 280,
        width: 260,
        height: 300,
        title: '🎨 视觉概念智能体 (Designer)',
        prompt: 'Futuristic warm living room, glowing smart home device hub in center, cozy modern aesthetics, photorealistic',
        model: 'Gemini 2.5 Flash',
        ratio: '16:9',
        status: 'idle'
      },
      {
        id: 'swarm-node-4',
        type: 'video-gen',
        x: 480,
        y: 680,
        width: 260,
        height: 190,
        title: '🎬 动态渲染智能体 (Animator)',
        prompt: 'Subtle light pulsing on a futuristic cozy smart home router, warm atmospheric lighting, high definition',
        model: 'Veo 2.0',
        ratio: '16:9',
        status: 'idle'
      }
    ],
    groups: [
      {
        id: 'swarm-group-1',
        title: '智能体协作集群 (Agent Swarm Cluster)',
        color: 'violet',
        x: 50,
        y: -10,
        width: 780,
        height: 1110,
        nodeIds: ['swarm-node-1', 'swarm-node-2', 'swarm-node-3', 'swarm-node-4']
      }
    ],
    connections: [
      { id: 'swarm-conn-1', fromNodeId: 'swarm-node-1', toNodeId: 'swarm-node-2' },
      { id: 'swarm-conn-2', fromNodeId: 'swarm-node-1', toNodeId: 'swarm-node-3' },
      { id: 'swarm-conn-3', fromNodeId: 'swarm-node-1', toNodeId: 'swarm-node-4' }
    ]
  },
  {
    id: 'sys-agent-debate',
    name: '智能体对抗辩论 (Dual Agent Debate)',
    description: '通过让正方与反方智能体进行论点交锋，最后由终审裁判进行总结评估，帮助我们更全面、无死角地做决策。',
    isSystem: true,
    category: 'collaboration',
    nodes: [
      {
        id: 'deb-node-1',
        type: 'text',
        x: 100,
        y: 240,
        width: 280,
        height: 200,
        title: '🎙️ 主持人 (Debate Moderator)',
        content: '【辩题】「AI 绘画是否会完全终结人类商业画师的职业生涯？」\n\n【规则】正反两方各举出 3 个核心依据，最后由裁判综合权衡。'
      },
      {
        id: 'deb-node-2',
        type: 'text',
        x: 460,
        y: 50,
        width: 280,
        height: 240,
        title: '⚡ 正方: 效率颠覆派 (AI Proponent)',
        content: '依据 1: 生成成本降低 99%，商业项目实现秒级交付。\n依据 2: 创意无限迭代，不再受限于人工手速与体力上限。\n依据 3: 完美适配标准化的快消品和素材背景设计。'
      },
      {
        id: 'deb-node-3',
        type: 'text',
        x: 460,
        y: 350,
        width: 280,
        height: 240,
        title: '🛡️ 反方: 人文守护派 (AI Skeptic)',
        content: '依据 1: AI 缺乏真实的情感与生命体验，作品流于同质化。\n依据 2: 版权与合法性争议悬而未决，高端品牌极力规避风险。\n依据 3: 客户沟通与深度定制、反复打磨的极致体验依然不可替代。'
      },
      {
        id: 'deb-node-4',
        type: 'text',
        x: 820,
        y: 240,
        width: 280,
        height: 220,
        title: '⚖️ 终审裁判 (Debate Judge)',
        content: '【裁判总结】：\n\nAI 在标准化、中低端快消品领域确实会大幅替代人工。然而，在具有深度战略价值、情感表达、高端艺术定制中，AI 只是画师手里的「超级画笔」。\n\n未来属于「掌握 AI 工具的画师」，而非 AI 自身。'
      }
    ],
    groups: [
      {
        id: 'deb-group-1',
        title: '多智能体评估对抗流 (Agent Adversarial Evaluation)',
        color: 'pink',
        x: 50,
        y: -10,
        width: 1110,
        height: 660,
        nodeIds: ['deb-node-1', 'deb-node-2', 'deb-node-3', 'deb-node-4']
      }
    ],
    connections: [
      { id: 'deb-conn-1', fromNodeId: 'deb-node-1', toNodeId: 'deb-node-2' },
      { id: 'deb-conn-2', fromNodeId: 'deb-node-1', toNodeId: 'deb-node-3' },
      { id: 'deb-conn-3', fromNodeId: 'deb-node-2', toNodeId: 'deb-node-4' },
      { id: 'deb-conn-4', fromNodeId: 'deb-node-3', toNodeId: 'deb-node-4' }
    ]
  },
  {
    id: 'sys-bento-brainstorm',
    name: '脑暴思维图谱 (Bento Brainstorm)',
    description: '基于不同颜色的便签（Sticky Notes）进行发散性的脑暴整理，结构轻盈直观，方便记录灵感。',
    isSystem: true,
    category: 'brainstorm',
    nodes: [
      {
        id: 'bento-node-1',
        type: 'text',
        x: 100,
        y: 260,
        width: 280,
        height: 180,
        title: '🎯 脑暴主题: 画布编辑器体验升级',
        content: '我们需要梳理并规划出能够让用户体验产生质变的产品特性。\n\n向右延展出各维度的脑暴便签。'
      },
      {
        id: 'bento-node-2',
        type: 'sticky',
        x: 460,
        y: 40,
        width: 200,
        height: 160,
        title: '💡 功能: 自动排版布局',
        content: '一键将散乱的卡片对齐为规整的网格或流向树，再多卡片也不会显得凌乱。',
        color: 'yellow'
      },
      {
        id: 'bento-node-3',
        type: 'sticky',
        x: 460,
        y: 220,
        width: 200,
        height: 160,
        title: '🎨 视觉: 磨砂玻璃拟物',
        content: '使用高雅的暗色调配霓虹荧光色，加入轻微的柔和阴影，科技感拉满。',
        color: 'pink'
      },
      {
        id: 'bento-node-4',
        type: 'sticky',
        x: 460,
        y: 400,
        width: 200,
        height: 160,
        title: '💾 效率: 进度快照历史',
        content: '允许用户随时给画布打上快照标签，不用担心误操作毁掉已经理顺的连线。',
        color: 'cyan'
      },
      {
        id: 'bento-node-5',
        type: 'sticky',
        x: 460,
        y: 580,
        width: 200,
        height: 160,
        title: '📚 预设: 经典模板库',
        content: '系统自带多个常用工作流模板，让新用户直接上手，不需要白手起家。',
        color: 'emerald'
      }
    ],
    groups: [],
    connections: [
      { id: 'bento-conn-1', fromNodeId: 'bento-node-1', toNodeId: 'bento-node-2' },
      { id: 'bento-conn-2', fromNodeId: 'bento-node-1', toNodeId: 'bento-node-3' },
      { id: 'bento-conn-3', fromNodeId: 'bento-node-1', toNodeId: 'bento-node-4' },
      { id: 'bento-conn-4', fromNodeId: 'bento-node-1', toNodeId: 'bento-node-5' }
    ]
  }
];

