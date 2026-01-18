import { type ReactNode, useState } from "react";

export type TooltipProps = {
  content: string;
  children: ReactNode;
};

export function Tooltip({ content, children }: TooltipProps) {
  const [isVisible, setIsVisible] = useState(false);

  return (
    <div
      className="relative inline-flex items-center outline-none select-none"
      onMouseEnter={() => setIsVisible(true)}
      onMouseLeave={() => setIsVisible(false)}
    >
      {children}
      {isVisible && (
        <div
          className="absolute bottom-full left-1/2 -translate-x-1/2 mb-2 px-4 py-2 bg-white text-stone-700 text-[11px] font-medium rounded-xl shadow-lg border border-stone-200 min-w-[180px] max-w-[280px] z-50 pointer-events-none transition-opacity duration-200"
        >
          {content.split('\n').map((line, i) => (
            <div key={i} className={i > 0 ? 'mt-1' : ''}>{line}</div>
          ))}
          {/* 小三角箭头 */}
          <div className="absolute top-full left-1/2 -translate-x-1/2 -mt-[1px] w-0 h-0 border-l-[6px] border-l-transparent border-r-[6px] border-r-transparent border-t-[6px] border-t-white drop-shadow-sm" />
        </div>
      )}
    </div>
  );
}
