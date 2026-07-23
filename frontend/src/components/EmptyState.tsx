import { Link } from "react-router-dom";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

type EmptyStateProps = {
  title: string;
  description?: string;
  actionLabel?: string;
  actionTo?: string;
  className?: string;
};

export function EmptyState({
  title,
  description,
  actionLabel,
  actionTo,
  className,
}: EmptyStateProps) {
  return (
    <div
      className={cn(
        "flex flex-col items-start gap-3 rounded-xl border border-dashed border-border bg-muted px-6 py-10",
        className,
      )}
    >
      <h3 className="text-title-md text-foreground">{title}</h3>
      {description && <p className="max-w-md text-sm text-muted-foreground">{description}</p>}
      {actionLabel && actionTo && (
        <Button asChild size="sm" className="mt-2">
          <Link to={actionTo}>{actionLabel}</Link>
        </Button>
      )}
    </div>
  );
}
