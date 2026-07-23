import * as React from "react";
import { cva, type VariantProps } from "class-variance-authority";
import { cn } from "@/lib/utils";

/* DESIGN.md badge-pill: surface-strong + caption-strong; status = text color only */
const badgeVariants = cva(
  "inline-flex items-center rounded-full border border-transparent px-3 py-1 text-caption-strong transition-colors focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-2",
  {
    variants: {
      variant: {
        default: "bg-secondary text-foreground",
        secondary: "bg-secondary text-secondary-foreground",
        destructive: "bg-transparent text-destructive",
        outline: "border-border bg-transparent text-foreground",
        success: "bg-transparent text-success",
        muted: "bg-muted text-muted-foreground",
      },
    },
    defaultVariants: {
      variant: "default",
    },
  },
);

export interface BadgeProps
  extends React.HTMLAttributes<HTMLDivElement>,
    VariantProps<typeof badgeVariants> {}

function Badge({ className, variant, ...props }: BadgeProps) {
  return <div className={cn(badgeVariants({ variant }), className)} {...props} />;
}

export { Badge, badgeVariants };

export function statusBadgeVariant(
  status: string,
): NonNullable<VariantProps<typeof badgeVariants>["variant"]> {
  const s = status.toLowerCase();
  if (s === "paid" || s === "completed" || s === "enabled") return "success";
  if (s === "failed" || s === "cancelled" || s === "canceled") return "destructive";
  if (s === "expired" || s === "disabled") return "muted";
  return "default";
}
