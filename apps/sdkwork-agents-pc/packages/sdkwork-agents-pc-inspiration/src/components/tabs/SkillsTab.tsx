import React from 'react';
import { Plus } from 'lucide-react';

import type { SkillCategory } from '../../types';

interface SkillsTabProps {
  filteredSkills: SkillCategory[];
}

export const SkillsTab: React.FC<SkillsTabProps> = ({ filteredSkills }) => {
  return (
    <div className="space-y-12">
      {filteredSkills.length === 0 ? (
        <div className="text-center py-20 text-zinc-500 text-sm">
          没有找到符合条件的创意技能
        </div>
      ) : (
        filteredSkills.map(category => (
          <div key={category.category} className="space-y-4">
            {/* Category Label */}
            <div className="flex items-center gap-2">
              <span className="w-1.5 h-4 bg-cyan-500 rounded-full"></span>
              <h2 className="text-[15px] font-bold text-zinc-100 tracking-wide">
                {category.category}
              </h2>
            </div>

            {/* Skill Cards Grid */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
              {category.items.map(skill => (
                <div 
                  key={skill.id}
                  className="bg-[#1e1e1e] border border-white/5 hover:border-white/10 hover:bg-[#222222] transition-all p-5 rounded-2xl relative flex flex-col justify-between h-[180px] group cursor-pointer"
                >
                  {/* Plus button inside top-right card */}
                  <button className="absolute top-4 right-4 w-7 h-7 rounded-full bg-white/5 hover:bg-white/10 text-zinc-400 hover:text-white flex items-center justify-center transition-all">
                    <Plus size={14} />
                  </button>

                  {/* Top part */}
                  <div>
                    <h3 className="text-[13.5px] font-bold text-zinc-200 pr-6 leading-normal group-hover:text-cyan-400 transition-colors">
                      {skill.title}
                    </h3>
                    <p className="text-[11.5px] text-zinc-400 mt-2 line-clamp-3 leading-relaxed">
                      {skill.desc}
                    </p>
                  </div>

                  {/* Bottom metadata */}
                  <div className="flex items-center justify-between border-t border-white/5 pt-3 mt-1.5">
                    <div className="flex items-center gap-1 text-[11px] text-zinc-500">
                      <span>❤️ {skill.likes}</span>
                      <span className="text-zinc-600">•</span>
                      <span className="truncate max-w-[90px]">来自 · {skill.author}</span>
                    </div>
                    <span className="text-[10px] text-zinc-500 bg-white/5 group-hover:bg-cyan-500/10 group-hover:text-cyan-400 px-2 py-0.5 rounded-full transition-all">
                      使用
                    </span>
                  </div>
                </div>
              ))}
            </div>
          </div>
        ))
      )}
    </div>
  );
};
