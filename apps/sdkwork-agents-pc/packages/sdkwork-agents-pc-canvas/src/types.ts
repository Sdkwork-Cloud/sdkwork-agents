export type NodeType = 'text' | 'image-gen' | 'video-gen' | 'flow-step' | 'sticky';

export interface CanvasNode {
  id: string;
  type: NodeType;
  x: number;
  y: number;
  width: number;
  height: number;
  title: string;
  content?: string; // For text/markdown card
  prompt?: string;  // For AI generation card
  model?: string;   // Active model selected
  ratio?: string;   // Image/Video aspect ratio
  status?: 'idle' | 'generating' | 'completed' | 'failed';
  progress?: number;
  mediaUrl?: string;
  duration?: number; // Video duration (s)
  count?: number; // Number of items to generate
  resolution?: string; // Video resolution (720P / 1080P)
  videoMode?: 'all_around' | 'first_last' | 'smart_multi'; // Video references mode
  refImageNodeId?: string; // Reference to source image node (for Image-to-Video)
  refTextNodeId?: string;  // Reference to source text/prompt node
  groupId?: string;        // ID of the group this node belongs to
  isCollapsed?: boolean;   // Whether the node is collapsed
  color?: string;          // Optional custom background/color (for sticky notes, etc)
  editorMode?: 'edit' | 'preview';
  fontStyle?: 'sans' | 'serif' | 'mono';
  showTOC?: boolean;
}

export interface CanvasGroup {
  id: string;
  title: string;
  color: 'cyan' | 'yellow' | 'pink' | 'emerald' | 'violet' | 'orange';
  x: number;
  y: number;
  width: number;
  height: number;
  nodeIds: string[];
  isCollapsed?: boolean;
}

export interface Connection {
  id: string;
  fromNodeId: string;
  toNodeId: string;
  controlPoint?: { x: number; y: number };
}

export type CanvasTool = 'select' | 'hand' | 'text' | 'image' | 'video';

export interface PanPosition {
  x: number;
  y: number;
}

