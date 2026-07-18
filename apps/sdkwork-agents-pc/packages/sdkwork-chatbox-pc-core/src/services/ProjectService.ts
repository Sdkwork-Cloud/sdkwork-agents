import { ChatSession } from '../sdk/types';
import { uuid } from '@sdkwork/utils';

export interface ProjectDetails {
  id: string;
  name: string;
  chats: ChatSession[];
}

export class ProjectService {
  /**
   * Fetch all projects
   */
  static async getProjects(): Promise<string[]> {
    await new Promise(resolve => setTimeout(resolve, 200));
    return [
      "机器人项目",
      "灵犀仙途",
      "桌面端项目标准",
      "移动端架构标准",
      "电商项目",
    ];
  }

  /**
   * Fetch project details including its chats
   */
  static async getProjectDetails(projectName: string): Promise<ProjectDetails> {
    await new Promise(resolve => setTimeout(resolve, 300));
    
    // Generate some mock chats for the project
    const mockChats: ChatSession[] = [
      {
        id: uuid(),
        title: `${projectName}规则`,
        messages: [
          { id: uuid(), role: 'user', text: '写一篇完整的角色文档，以独立的markdown输出。' }
        ],
        updatedAt: Date.now() - 1000 * 60 * 60 * 24 * 2 // 2 days ago
      },
      {
        id: uuid(),
        title: `讨论: ${projectName}架构`,
        messages: [
          { id: uuid(), role: 'user', text: '我们应该如何设计架构？' }
        ],
        updatedAt: Date.now() - 1000 * 60 * 60 * 24 * 5 // 5 days ago
      }
    ];

    return {
      id: projectName,
      name: projectName,
      chats: mockChats
    };
  }
}
