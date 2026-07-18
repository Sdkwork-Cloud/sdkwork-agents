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
}

export interface DiscoverData {
  banner: DiscoverItem;
  cols: DiscoverItem[][];
}

export const MOCK_SHORT_VIDEOS: ShortVideo[] = [
  {
    id: "v-1",
    title: "原创AI短片《认证》",
    author: "活圣圣",
    avatar: "https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=100&q=80",
    likes: 1358,
    duration: "05:52",
    desc: "一个没有通过算法认证的老人，在风雪、羊群、土地和自己的沉默中...",
    cover: "https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=800&q=80",
    videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-space-exploration-with-a-retro-futuristic-computer-43180-large.mp4"
  },
  {
    id: "v-2",
    title: "《百匠：纸契灵》",
    author: "阿生的ai",
    avatar: "https://images.unsplash.com/photo-1599566150163-29194dcaad36?w=100&q=80",
    likes: 64,
    duration: "08:19",
    desc: "异族可借风沙掩星辰，百匠合一为侍魂。",
    cover: "https://images.unsplash.com/photo-1514539079130-25950c84af65?w=800&q=80",
    videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-girl-running-on-the-wet-grass-at-sunrise-44754-large.mp4"
  },
  {
    id: "v-3",
    title: "原创AI动画《阿房宫》",
    author: "灵湘_SSS",
    avatar: "https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=100&q=80",
    likes: 492,
    duration: "15:46",
    desc: "工匠少年阿更铤而走险夜闯阿房宫，只为拯救病危的阿母。穿过六国...",
    cover: "https://images.unsplash.com/photo-1518005020951-eccb494ad742?w=800&q=80",
    videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-forest-stream-in-the-sunlight-529-large.mp4"
  },
  {
    id: "v-4",
    title: "人，橘咪带你逛上海卢浮宫大展!",
    author: "海辛",
    avatar: "https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=100&q=80",
    likes: 1680,
    duration: "01:17",
    desc: "我们为卢浮宫在 @浦东美术馆 的上海首展又做了一支官方宣传片，橘咪指路...",
    cover: "https://images.unsplash.com/photo-1514888286974-6c03e2ca1dba?w=800&q=80",
    videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-pink-neon-running-shoes-in-slow-motion-44583-large.mp4"
  },
  {
    id: "v-5",
    title: "这一面《镜子》照见了我们大多数人的一生",
    author: "我是楚墨R®",
    avatar: "https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=100&q=80",
    likes: 237,
    duration: "04:07",
    desc: "一面镜子，照过初生，照过成长，照过出嫁，也照过老去。我们以为...",
    cover: "https://images.unsplash.com/photo-1490730141103-6cac27aaab94?w=800&q=80",
    videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-stars-in-space-background-1611-large.mp4"
  },
  {
    id: "v-6",
    title: "《生命》",
    author: "子和",
    avatar: "https://images.unsplash.com/photo-1522075469751-3a6694fb2f61?w=100&q=80",
    likes: 278,
    duration: "03:21",
    desc: "弹幕如彩虹般划过眼前。在这个信息满溢的时代，我们日复一日注视...",
    cover: "https://images.unsplash.com/photo-1470071459604-3b5ec3a7fe05?w=800&q=80",
    videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-spinning-gold-coins-on-dark-surface-44675-large.mp4"
  },
  {
    id: "v-7",
    title: "Electronic night",
    author: "Yea野了",
    avatar: "https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=100&q=80",
    likes: 614,
    duration: "00:27",
    desc: "在这片迷离的城市中，夜晚才是真正的开始，音乐与光影交织，迷幻...",
    cover: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=800&q=80",
    videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-abstract-glowing-lines-loop-45244-large.mp4"
  },
  {
    id: "v-8",
    title: "《渊》",
    author: "BackMan",
    avatar: "https://images.unsplash.com/photo-1507003211169-0a1dd7228f2d?w=100&q=80",
    likes: 137,
    duration: "09:26",
    desc: "人类从不放弃！ 第一次尝试手接9分钟的AI短片，有很多不足的地方...",
    cover: "https://images.unsplash.com/photo-1447752875215-b2761acb3c5d?w=800&q=80",
    videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-hands-holding-and-showing-a-retro-cassette-tape-43176-large.mp4"
  },
  {
    id: "v-9",
    title: "崔健《快让我在这雪地上撒点野》MV",
    author: "LH品牌设计",
    avatar: "https://images.unsplash.com/photo-1521119989659-a83eee488004?w=100&q=80",
    likes: 20,
    duration: "07:40",
    desc: "当痛房的寂静被古筝击碎，一个灵魂脱颖而脱，赤脚奔向风雪...",
    cover: "https://images.unsplash.com/photo-1486520299386-6d106b22014b?w=800&q=80",
    videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-waterfall-in-forest-2213-large.mp4"
  },
  {
    id: "v-10",
    title: "《怀民亦未寝》邵氏武侠版",
    author: "AI小宝",
    avatar: "https://images.unsplash.com/photo-1544005313-94ddf0286df2?w=100&q=80",
    likes: 420,
    duration: "01:38",
    desc: "语文课本最疯的一夜 | 邵氏武侠版承天寺夜游。#邵氏电影 #武侠 #语文",
    cover: "https://images.unsplash.com/photo-1441974231531-c6227db76b6e?w=800&q=80",
    videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-waves-in-the-ocean-near-shore-43026-large.mp4"
  }
];

export const MOCK_SKILLS: SkillCategory[] = [
  {
    category: "短剧影视",
    items: [
      {
        id: "s-1",
        title: "叙事短片导演分镜",
        desc: "以淼海荧光导演分镜方法论产出故事短片：导演意图书、九列分镜表、4~15s Clip 表与逐 Clip 提示词，直至成片。当用户要做...",
        likes: 162,
        author: "森海荧光"
      },
      {
        id: "s-2",
        title: "一图成片-电影广告全能导演",
        desc: "本技能基于用户上传的一张或多张图片，生成电影短片、电影预告、TVC广告、品牌广告、产品广告、社媒短片、IP角色预告、...",
        likes: 107,
        author: "渊静-中意"
      },
      {
        id: "s-3",
        title: "剧本宽视频一条龙创作",
        desc: "本技能从创意/原始文本出发，一站式完成标准剧本转换、视觉资产提取、设定图生成、分镜视频设计、资产匹配到最终视频成...",
        likes: 90,
        author: "娜乌斯嘉"
      },
      {
        id: "s-4",
        title: "世界观美术设定",
        desc: "通用美术设定/世界观skill（跨项目通用，与任何具体项目解耦）。负责世界与场景的内容设计：世界观法则/世界规则/系...",
        likes: 38,
        author: "慕影-中意"
      },
      {
        id: "s-5",
        title: "AI演员表情导演",
        desc: "本技能通过简单交互生成符合影视要求的音画轴表情提示词，支持批量生成表情视频，当用户需要为AI视频制作表情精微...",
        likes: 68,
        author: "即梦AI"
      }
    ]
  },
  {
    category: "电商营销",
    items: [
      {
        id: "s-6",
        title: "顶级波普视觉广告导演",
        desc: "本技能输出12秒高频闪切快消品短视频分镜表。专注Y2K/复古潮酷风格创作。当用户提供快消产品信息、视觉元素和氛围关键...",
        likes: 58,
        author: "即梦AI"
      },
      {
        id: "s-7",
        title: "珠宝电商图文视频一站式",
        desc: "仅限珠宝品类（戒指/项链/耳环/手镯/吊坠/胸针等）的电商素材一站式生成。图片、文案、视频一体化交付：按平台预设产...",
        likes: 36,
        author: "地质大学博士说AI"
      },
      {
        id: "s-8",
        title: "爆款电商短视频题材创意生成",
        desc: "电商短视频创作助手，支持多题材选择、时长配置，每个题材生成5组差异化方案，已集成爆款短视频高完播创作规则",
        likes: 25,
        author: "Carson"
      },
      {
        id: "s-9",
        title: "反差叙事剧情广告",
        desc: "本技能根据用户提供的产品、卖点、目标人群和使用场景，生成15秒反差广告的完整叙事方案。技能不采用普通“痛点-产品-...",
        likes: 13,
        author: "话神闲"
      },
      {
        id: "s-10",
        title: "时尚走秀短视频导演",
        desc: "本技能将简单输入升维拆解为15秒充满高级感的工业级奢侈品时尚走秀短视频脚本。当用户提供模特特征、服装特征和场景，...",
        likes: 33,
        author: "即梦AI"
      }
    ]
  },
  {
    category: "创意艺术",
    items: [
      {
        id: "s-11",
        title: "系列套图生成",
        desc: "本技能用于将用户提供的母版提示词、参考图、产品图、角色设定、品牌资料、Logo、IP形象或视觉方向，抽象为稳定的系列...",
        likes: 44,
        author: "渊静-中意"
      },
      {
        id: "s-12",
        title: "名号十五秒视频风格引擎",
        desc: "本技能从参考图提取底层视觉基因与导演写法，保存为结构化JSON资产，并自动将一句话需求转换为十五秒三段式、带感官...",
        likes: 25,
        author: "即梦AI"
      },
      {
        id: "s-13",
        title: "珠宝设计出款",
        desc: "专业珠宝设计与批量出款技能。面向戒指、项链、耳环、手镯、吊坠、胸针及高级珠宝系列，支持从文字描述、风格参考、产品...",
        likes: 15,
        author: "地质大学博士说AI"
      },
      {
        id: "s-14",
        title: "珠宝设计进化",
        desc: "选择器驱动的一次性珠宝设计进化技能。用户上传底库珠宝图/草图/元素图文描述初雏形后，通过单选/多选收集结构保留...",
        likes: 12,
        author: "地质大学博士说AI"
      },
      {
        id: "s-15",
        title: "角色精灵图动画产线",
        desc: "端到端生成游戏角色像素帧动画视频。核心调用 multi_modal2video (全能参考)：角色图作为@图片1被提示词引用，构图与占比...",
        likes: 20,
        author: "AIGC炼丹师"
      }
    ]
  }
];

export const MOCK_ACTIVITIES: Activity[] = [
  {
    id: "act-1",
    title: "第38届大众电影百花奖配套活动·AIGC推荐单元",
    desc: "面向全球征集AI影像短片，发掘兼具技术突破与电影艺术价值的精品...",
    status: "距离截稿还有10天13小时",
    tag: "直通百花、线下展映交流",
    participants: 613,
    cover: "https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?w=800&q=80",
    banner: "https://images.unsplash.com/photo-1489599849927-2ee91cede3ba?w=1200&q=80",
    timeRange: "2026-06-30 00:00:00 - 2026-07-20 23:59:59",
    background: "大众电影百花奖是由中国文学艺术界联合会和中国电影家协会主办的全国性、群众性评奖项目。首届大众电影百花奖于1962年在北京颁发，第38届大众电影百花奖系列活动将于2026年在北京举办。\n\nAIGC推荐单元首次展现于第38届大众电影百花奖系列活动中，联合即梦 AI 作为第38届大众电影百花奖系列活动·AIGC技术合作伙伴，倾力打造极具行业影响力的 AIGC 推荐活动。本活动以“光影新纪元，AI 创未来”为核心理念，面向全球创作者广泛征集优质 AI 影像短片，致力于鼓励广大创作者大胆探索 AI 在影视创作中的创新应用，深耕内容创作，挖掘兼具真实技术突破与优质电影艺术价值的精品影像内容。",
    works: [
      {
        id: "w-1",
        title: "《临水》阿嬷说,王爷爷的大船会载着阿玛回家",
        author: "YoRHa",
        avatar: "https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=100&q=80",
        likes: 72,
        duration: "06:53",
        cover: "https://images.unsplash.com/photo-1518005020951-eccb494ad742?w=800&q=80",
        videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-space-exploration-with-a-retro-futuristic-computer-43180-large.mp4",
        desc: "“天乌乌，海茫茫，老船载亲转回乡。”《临水》的故事..."
      },
      {
        id: "w-2",
        title: "《荔镜缘》",
        author: "Yizen",
        avatar: "https://images.unsplash.com/photo-1599566150163-29194dcaad36?w=100&q=80",
        likes: 41,
        duration: "15:47",
        cover: "https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=800&q=80",
        videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-girl-running-on-the-wet-grass-at-sunrise-44754-large.mp4",
        desc: "《荔镜缘》是一部时长15分47秒的纯AIGC国漫3D..."
      },
      {
        id: "w-3",
        title: "北影节舞蹈影像艺术大赏AIGC单元【最佳影片】《奔灵》",
        author: "九森LU",
        avatar: "https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=100&q=80",
        likes: 34,
        duration: "10:00",
        cover: "https://images.unsplash.com/photo-1514539079130-25950c84af65?w=800&q=80",
        videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-forest-stream-in-the-sunlight-529-large.mp4",
        desc: "AIGC舞蹈短片电影《奔灵》，以不可再生生态保..."
      },
      {
        id: "w-4",
        title: "同性人",
        author: "未目同行",
        avatar: "https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=100&q=80",
        likes: 26,
        duration: "12:06",
        cover: "https://images.unsplash.com/photo-1500648767791-00dcc994a43e?w=100&q=80",
        videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-pink-neon-running-shoes-in-slow-motion-44583-large.mp4",
        desc: "同性人，有时被称为阴阳人或双性人，指那些出..."
      },
      {
        id: "w-5",
        title: "大胡子",
        author: "AIGC佬猫",
        avatar: "https://images.unsplash.com/photo-1522075469751-3a6694fb2f61?w=100&q=80",
        likes: 24,
        duration: "02:41",
        cover: "https://images.unsplash.com/photo-1511671782779-c97d3d27a1d4?w=800&q=80",
        videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-stars-in-space-background-1611-large.mp4",
        desc: "一个所有孩子都必须留大胡子的灰色城市里，男..."
      },
      {
        id: "w-6",
        title: "《白日梦》AIGC短片",
        author: "Ayahara绫原",
        avatar: "https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=100&q=80",
        likes: 22,
        duration: "09:51",
        cover: "https://images.unsplash.com/photo-1470071459604-3b5ec3a7fe05?w=800&q=80",
        videoUrl: "https://assets.mixkit.co/videos/preview/mixkit-spinning-gold-coins-on-dark-surface-44675-large.mp4",
        desc: "长大后在杭州工作了多年的丁妍，几乎没有在杭州..."
      }
    ]
  },
  {
    id: "act-2",
    title: "健力宝来电 AI 整活大赛",
    desc: "用AI视频演绎「含气电解质 劲爽更尽兴」",
    status: "距离截稿还有11天13小时",
    tag: "本期活动设置 20万 奖金池和15万抖音流量池",
    participants: 775,
    cover: "https://images.unsplash.com/photo-1527960656366-ee2a999e32e6?w=800&q=80",
    banner: "https://images.unsplash.com/photo-1527960656366-ee2a999e32e6?w=1200&q=80",
    timeRange: "2026-07-01 00:00:00 - 2026-07-21 23:59:59",
    background: "健力宝携手即梦 AI 开启 AI 整活视频大赛！用你天马行空的想象力，配合 AI 工具演绎充满活力的酷爽体验，获取超额现金激励及全域流量扶持！",
    works: []
  },
  {
    id: "act-3",
    title: "雀巢咖啡丝滑拿铁一口重启 AI创作大赛",
    desc: "你的节奏 看你的",
    status: "距离截稿还有17天13小时",
    tag: "本期活动设置20万奖金池和15万抖音流量池",
    participants: 324,
    cover: "https://images.unsplash.com/photo-1495474472287-4d71bcdd2085?w=800&q=80",
    banner: "https://images.unsplash.com/photo-1495474472287-4d71bcdd2085?w=1200&q=80",
    timeRange: "2026-07-05 00:00:00 - 2026-07-27 23:59:59",
    background: "雀巢咖啡经典丝滑拿铁与即梦 AI 强强联合，带来‘一口重启’的主题 AI 创意大赛。不管你是新手创作者还是资深剪辑大师，欢迎在此展现你的精彩世界！",
    works: []
  },
  {
    id: "act-4",
    title: "抖音 AI 创作大赛",
    desc: "抖音 AI 创作大赛火热启幕，即梦 AI 作为深度合作伙伴全程助力!",
    status: "距离截稿还有41天13小时",
    tag: "400万现金与2000万即梦积分已就位",
    participants: 6122,
    cover: "https://images.unsplash.com/photo-1516280440614-37939bbacd6a?w=800&q=80",
    banner: "https://images.unsplash.com/photo-1516280440614-37939bbacd6a?w=1200&q=80",
    timeRange: "2026-06-20 00:00:00 - 2026-08-20 23:59:59",
    background: "抖音联合多家头部 AI 生产力工具开启首届 AI 创作大赛，高达400万的总奖金与大牌展映机会，邀你一起见证属于 AIGC 时代的艺术爆发！",
    works: []
  },
  {
    id: "act-5",
    title: "2026大学生AI艺术季·AI影像创作单元",
    desc: "即梦AI携手2026大学生艺术季打造AI影像创作单元",
    status: "距离截稿还有5天13小时",
    tag: "2026大学生艺术季组委会一等奖",
    participants: 814,
    cover: "https://images.unsplash.com/photo-1620311497210-67ee56d11e5c?w=800&q=80",
    banner: "https://images.unsplash.com/photo-1620311497210-67ee56d11e5c?w=1200&q=80",
    timeRange: "2026-06-15 00:00:00 - 2026-07-15 23:59:59",
    background: "为了促进高校学子的 AI 融合创作，2026大学生艺术季组委会联合即梦 AI 共同发布官方影像单元，挖掘新生代的高校 AI 艺术大师！",
    works: []
  }
];

export const FEATURE_CARDS = [
  { id: 'octo', title: 'Octo', desc: 'Vibe create, 创作自然流动', tag: 'Beta', icon: '✨', bg: 'bg-gradient-to-br from-orange-400 to-rose-400' },
  { id: 'canvas', title: '无限画布', desc: '自由创作', icon: '🎨', bg: 'bg-gradient-to-br from-blue-400 to-cyan-400' },
  { id: 'agent', title: 'Agent 模式', desc: '52.0视频创作', icon: '🤖', bg: 'bg-gradient-to-br from-emerald-400 to-teal-400' },
  { id: 'image', title: '图片生成', desc: '智能美学提升', tag: 'New', icon: '🖼️', bg: 'bg-gradient-to-br from-blue-500 to-indigo-500' },
  { id: 'video', title: '视频生成', desc: 'Seedance 2.0', icon: '🎬', bg: 'bg-gradient-to-br from-purple-500 to-violet-500' },
];

export const DISCOVER_MOCK_DATA = {
  banner: {
    id: 'banner',
    src: "https://images.unsplash.com/photo-1620311497210-67ee56d11e5c?w=1200&q=80",
    alt: "banner",
    title: "2026大学生AI艺术季·AI影像创作单元",
    prompt: "2026大学生AI艺术季·AI影像创作单元 官方海报, 科幻梦幻极简微缩立体风格",
    author: "官方团队",
    avatar: "https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=100&q=80",
    date: "2026-07-09",
    likes: 812,
    aspectRatio: "21:9",
    model: "4.0"
  },
  cols: [
    [ // col 1
      { id: 'c1-1', src: "https://images.unsplash.com/photo-1544365558-35aa4afcf11f?w=600&q=80", alt: "girl grass", author: "小明", avatar: "https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=100&q=80", likes: 128, prompt: "Girl running in the sun on lush green hills, realistic light, 8k" },
      { id: 'c1-2', src: "https://images.unsplash.com/photo-1529626455594-4ff0802cfb7e?w=600&q=80", alt: "girl car", author: "Alice", avatar: "https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=100&q=80", likes: 45, prompt: "A cinematic portrait of a woman looking out of a classic car window, warm golden hour tones" }
    ],
    [ // col 2
      { id: 'c2-1', src: "https://images.unsplash.com/photo-1516684732162-798a0062be99?w=600&q=80", alt: "girl sweater", author: "Bob", avatar: "https://images.unsplash.com/photo-1599566150163-29194dcaad36?w=100&q=80", likes: 89, prompt: "Cozy portrait of a young man wearing a wool sweater, cinematic atmosphere" },
      { id: 'c2-2', src: "https://images.unsplash.com/photo-1524504388940-b1c1722653e1?w=600&q=80", alt: "girl purple", author: "Charlie", avatar: "https://images.unsplash.com/photo-1527980965255-d3b416303d12?w=100&q=80", likes: 234, prompt: "Creative portrait with neon purple lighting, cybertech style, highly detailed" }
    ],
    [ // col 3
      { id: 'c3-1', src: "https://images.unsplash.com/photo-1707343843437-caacff5cfa74?w=600&q=80", alt: "monkey", author: "韩啸", avatar: "https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=100&q=80", likes: 31, title: "Adam Riches风格作品，妖魔化孙悟空...", prompt: "Adam Riches illustration style, demonized Sun Wukong character, high contrast ink sketch" },
      { id: 'c3-2', src: "https://images.unsplash.com/photo-1580477651161-55db29a3e6bb?w=600&q=80", alt: "corn", author: "David", avatar: "https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=100&q=80", likes: 56, prompt: "Golden corn field in late autumn, stunning landscape photography, 4k" }
    ],
    [ // col 4
      { id: 'c4-1', src: "https://images.unsplash.com/photo-1550684376-efcb96075908?w=600&q=80", alt: "fish tail", author: "元宝", avatar: "https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=100&q=80", likes: 116, title: "FISH TAIL ELEGANCE", prompt: "A glowing fish tail under deep blue neon waters, magical realistic render" },
      { id: 'c4-2', src: "https://images.unsplash.com/photo-1514888286974-6c03e2ca1dba?w=600&q=80", alt: "black cat", author: "Eve", avatar: "https://images.unsplash.com/photo-1599566150163-29194dcaad36?w=100&q=80", likes: 890, prompt: "Fluffy black cat with glowing green eyes, mystical atmosphere, realistic macro fur detail" }
    ],
    [ // col 5
      { id: 'c5-1', src: "https://images.unsplash.com/photo-1550684848-fac1c5b4e853?w=600&q=80", alt: "girl cat", author: "Frank", avatar: "https://images.unsplash.com/photo-1527980965255-d3b416303d12?w=100&q=80", likes: 12, prompt: "Cinematic medium shot of a cute girl cuddling with a white kitten, warm sunrise backlight" },
      { id: 'c5-2', src: "https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=600&q=80", alt: "girl accessories", author: "Grace", avatar: "https://images.unsplash.com/photo-1438761681033-6461ffad8d80?w=100&q=80", likes: 445, prompt: "An artistic studio portrait of a model with pearl facial decorations, high fashion concept" }
    ],
    [ // col 6
      { id: 'c6-1', src: "https://images.unsplash.com/photo-1515347619252-8706bf953724?w=600&q=80", alt: "girl traditional", author: "Harry", avatar: "https://images.unsplash.com/photo-1535713875002-d1d0cf377fde?w=100&q=80", likes: 77, prompt: "Portrait of a woman wearing traditional elegant attire, soft moody lighting" },
      { id: 'c6-2', src: "https://images.unsplash.com/photo-1506794778202-cad84cf45f1d?w=600&q=80", alt: "man suit", author: "Ivy", avatar: "https://images.unsplash.com/photo-1494790108377-be9c29b29330?w=100&q=80", likes: 33, prompt: "Sharp studio portrait of a man in a bespoke navy suit, rich colors, professional grade" }
    ]
  ]
};
