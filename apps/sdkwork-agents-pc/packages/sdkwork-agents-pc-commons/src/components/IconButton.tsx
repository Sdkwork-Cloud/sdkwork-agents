import type { ButtonHTMLAttributes } from "react";

import { cn } from "../utils";

interface IconButtonProps extends ButtonHTMLAttributes<HTMLButtonElement> {
  active?: boolean;
}

export function IconButton({ className, active, type = "button", ...props }: IconButtonProps) {
  return (
    <button
      type={type}
      className={cn(
        "flex items-center justify-center w-[44px] h-[44px] rounded-lg text-[#86909c] transition-all duration-200 hover:scale-105 active:scale-90 hover:bg-white/10 hover:text-white",
        active && "bg-white/10 text-white",
        className,
      )}
      {...props}
    />
  );
}
