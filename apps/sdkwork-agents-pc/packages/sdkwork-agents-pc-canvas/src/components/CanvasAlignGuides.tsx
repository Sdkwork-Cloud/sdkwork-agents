import React from 'react';
import { cn } from '@sdkwork/agents-pc-commons';

interface AlignGuide {
  id: string;
  type: 'h' | 'v'; // h = horizontal line, v = vertical line
  coord: number;   // target axis coordinate
  start: number;   // bounding box line start
  end: number;     // bounding box line end
  label?: string;  // descriptive label
}

interface CanvasAlignGuidesProps {
  guides: AlignGuide[];
}

export const CanvasAlignGuides: React.FC<CanvasAlignGuidesProps> = ({ guides }) => {
  if (!guides || guides.length === 0) return null;

  return (
    <>
      {guides.map(guide => (
        <div
          key={guide.id}
          style={{
            position: 'absolute',
            left: guide.type === 'v' ? guide.coord : guide.start,
            top: guide.type === 'h' ? guide.coord : guide.start,
            width: guide.type === 'v' ? '1px' : (guide.end - guide.start),
            height: guide.type === 'h' ? '1px' : (guide.end - guide.start),
            zIndex: 45,
            pointerEvents: 'none'
          }}
          className={cn(
            "border-dashed transition-opacity duration-150 flex items-center justify-center",
            guide.type === 'v' 
              ? "border-l border-cyan-400/85 shadow-[0_0_8px_rgba(34,211,238,0.4)]" 
              : "border-t border-cyan-400/85 shadow-[0_0_8px_rgba(34,211,238,0.4)]"
          )}
        >
          {guide.label && (
            <span className="absolute px-1.5 py-0.5 rounded bg-cyan-950/95 border border-cyan-400/30 text-cyan-300 text-[9px] font-bold whitespace-nowrap scale-90 translate-x-4 -translate-y-4 select-none shadow-md font-sans">
              {guide.label}
            </span>
          )}
        </div>
      ))}
    </>
  );
};
