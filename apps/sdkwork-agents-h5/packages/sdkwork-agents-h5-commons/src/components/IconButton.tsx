import React from "react";
import { motion } from "motion/react";

import { cn } from "../utils";

interface IconButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  active?: boolean;
}

export const IconButton = React.forwardRef<HTMLButtonElement, IconButtonProps>(
  ({ className, active, ...props }, ref) => {
    return (
      <motion.button
        ref={ref as React.Ref<HTMLButtonElement>}
        whileHover={{ scale: 1.05 }}
        whileTap={{ scale: 0.9 }}
        className={cn(
          "flex items-center justify-center w-[44px] h-[44px] rounded-lg text-[#86909c] transition-all duration-200 hover:bg-white/10 hover:text-white",
          active && "bg-white/10 text-white",
          className,
        )}
        {...(props as React.ComponentProps<typeof motion.button>)}
      />
    );
  },
);
IconButton.displayName = "IconButton";
