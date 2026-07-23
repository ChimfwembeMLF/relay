import * as React from "react";
import { Slot } from "@radix-ui/react-slot";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

const buttonVariants = cva(
  "inline-flex items-center justify-center gap-2 whitespace-nowrap text-[16px] font-semibold leading-[1.15] transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-offset-2 disabled:pointer-events-none [&_svg]:pointer-events-none [&_svg]:size-4 [&_svg]:shrink-0",
  {
    variants: {
      variant: {
        default:
          "rounded-full bg-primary text-primary-foreground shadow-none hover:bg-primary-active disabled:bg-primary-disabled disabled:text-primary-foreground disabled:opacity-100",
        destructive:
          "rounded-full bg-destructive text-destructive-foreground shadow-none hover:bg-destructive/90 disabled:opacity-50",
        outline:
          "rounded-full border border-input bg-background shadow-none hover:bg-accent hover:text-accent-foreground disabled:opacity-50",
        secondary:
          "rounded-full bg-secondary text-secondary-foreground shadow-none hover:bg-secondary/80 disabled:opacity-50",
        ghost: "rounded-full hover:bg-accent hover:text-accent-foreground disabled:opacity-50",
        link: "rounded-none text-primary underline-offset-4 hover:underline disabled:opacity-50",
      },
      size: {
        default: "h-11 px-5 py-3",
        sm: "h-9 rounded-full px-4 text-sm",
        lg: "h-14 rounded-full px-8 text-base",
        icon: "h-11 w-11 rounded-full",
      },
    },
    defaultVariants: {
      variant: "default",
      size: "default",
    },
  },
);

export interface ButtonProps
  extends React.ButtonHTMLAttributes<HTMLButtonElement>,
    VariantProps<typeof buttonVariants> {
  asChild?: boolean;
}

const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className, variant, size, asChild = false, ...props }, ref) => {
    const Comp = asChild ? Slot : "button";
    return (
      <Comp className={cn(buttonVariants({ variant, size, className }))} ref={ref} {...props} />
    );
  },
);
Button.displayName = "Button";

export { Button, buttonVariants };
