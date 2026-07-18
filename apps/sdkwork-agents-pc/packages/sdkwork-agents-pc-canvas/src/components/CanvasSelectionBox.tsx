import React from 'react';

interface CanvasSelectionBoxProps {
  selectionBox: {
    startX: number;
    startY: number;
    currentX: number;
    currentY: number;
  } | null;
}

export const CanvasSelectionBox: React.FC<CanvasSelectionBoxProps> = ({ selectionBox }) => {
  if (!selectionBox) return null;

  if (
    isNaN(selectionBox.startX) ||
    isNaN(selectionBox.startY) ||
    isNaN(selectionBox.currentX) ||
    isNaN(selectionBox.currentY)
  ) return null;

  const left = Math.min(selectionBox.startX, selectionBox.currentX);
  const top = Math.min(selectionBox.startY, selectionBox.currentY);
  const width = Math.abs(selectionBox.currentX - selectionBox.startX);
  const height = Math.abs(selectionBox.currentY - selectionBox.startY);

  if (isNaN(left) || isNaN(top) || isNaN(width) || isNaN(height)) return null;

  // Don't render extremely tiny boxes
  if (width < 3 || height < 3) return null;

  return (
    <>
      <style>{`
        @keyframes selection-ants {
          to {
            stroke-dashoffset: -10;
          }
        }
        .animate-selection-ants {
          stroke-dasharray: 6, 4;
          animation: selection-ants 0.3s linear infinite;
        }
      `}</style>
      <div
        style={{
          left,
          top,
          width,
          height,
          zIndex: 100
        }}
        className="absolute bg-cyan-400/[0.04] rounded-lg pointer-events-none shadow-[0_0_30px_rgba(34,211,238,0.15)] border border-cyan-400/25 overflow-hidden transition-all duration-75"
      >
        <svg className="absolute inset-0 w-full h-full">
          <rect
            x="0.75"
            y="0.75"
            width="calc(100% - 1.5px)"
            height="calc(100% - 1.5px)"
            fill="none"
            stroke="#22d3ee"
            strokeWidth="1.5"
            className="animate-selection-ants"
            rx="6"
            ry="6"
          />
        </svg>
      </div>
    </>
  );
};
