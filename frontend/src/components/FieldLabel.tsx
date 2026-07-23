import { CircleHelp } from "lucide-react";
import { Label } from "@/components/ui/label";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";

type FieldLabelProps = {
  htmlFor: string;
  children: React.ReactNode;
  tip: string;
};

/** Label with a help icon tooltip (hover / focus / tap). */
export function FieldLabel({ htmlFor, children, tip }: FieldLabelProps) {
  return (
    <div className="flex items-center gap-1.5">
      <Label htmlFor={htmlFor}>{children}</Label>
      <Tooltip>
        <TooltipTrigger asChild>
          <button
            type="button"
            className="inline-flex size-5 shrink-0 items-center justify-center rounded-full text-muted-foreground transition-colors hover:text-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring"
            aria-label={`About ${typeof children === "string" ? children : "this field"}`}
          >
            <CircleHelp className="size-3.5" strokeWidth={2} />
          </button>
        </TooltipTrigger>
        <TooltipContent side="top" align="start">
          {tip}
        </TooltipContent>
      </Tooltip>
    </div>
  );
}
