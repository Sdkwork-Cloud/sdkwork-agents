import React from 'react';
import { Clock, Award, Users } from 'lucide-react';
import type { Activity } from '../../types';

interface ActivitiesTabProps {
  filteredActivities: Activity[];
  onSelectActivity: (activity: Activity) => void;
}

export const ActivitiesTab: React.FC<ActivitiesTabProps> = ({ filteredActivities, onSelectActivity }) => {
  return (
    <div>
      {filteredActivities.length === 0 ? (
        <div className="text-center py-20 text-zinc-500 text-sm">
          没有找到相关的赛事活动
        </div>
      ) : (
        <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-5">
          {filteredActivities.map(activity => (
            <div 
              key={activity.id}
              className="bg-[#1e1e1e] border border-white/5 hover:border-white/10 hover:bg-[#222222] rounded-2xl overflow-hidden group cursor-pointer transition-all flex flex-col h-[280px]"
              onClick={() => onSelectActivity(activity)}
            >
              {/* Top Wide Cover image of Activity */}
              <div className="relative h-[140px] w-full bg-zinc-900 overflow-hidden shrink-0">
                <img 
                  src={activity.cover} 
                  alt={activity.title} 
                  className="w-full h-full object-cover transition-transform duration-700 group-hover:scale-105"
                />
                <div className="absolute inset-0 bg-gradient-to-t from-black/80 to-transparent" />
                
                {/* Countdown / Status Tag in top-left */}
                <div className="absolute top-3 left-3 bg-yellow-500 text-black text-[10px] font-bold px-2.5 py-1 rounded-md flex items-center gap-1 shadow">
                  <Clock size={11} />
                  {activity.status}
                </div>
              </div>

              {/* Content Body */}
              <div className="p-5 flex-1 flex flex-col justify-between">
                <div>
                  <h3 className="text-[13.5px] font-bold text-zinc-100 line-clamp-1 group-hover:text-cyan-400 transition-colors leading-snug">
                    {activity.title}
                  </h3>
                  <p className="text-[11.5px] text-zinc-400 mt-1.5 line-clamp-2 leading-relaxed">
                    {activity.desc}
                  </p>
                </div>

                {/* Footer Details */}
                <div className="flex items-center justify-between border-t border-white/5 pt-3 mt-2 text-[11px] text-zinc-500">
                  <div className="flex items-center gap-1 text-yellow-500 font-medium">
                    <Award size={12} />
                    <span className="truncate max-w-[200px]">{activity.tag}</span>
                  </div>
                  <span className="shrink-0 text-zinc-400 flex items-center gap-1 font-mono">
                    <Users size={11} />
                    已有 {activity.participants} 人参与
                  </span>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
};
