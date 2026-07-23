import { cn } from "@/lib/utils";
import logoBlue from "@/assets/logo-blue.png";
import logoLight from "@/assets/logo.png";
import favicon from "@/assets/favicon.svg";

type BrandLogoProps = {
  className?: string;
  /** Image height/width classes */
  imgClassName?: string;
  alt?: string;
};

/** Tekrem wordmark — keep assets; color system is Coinbase Blue via theme. */
export function BrandLogo({
  className,
  imgClassName = "h-8 w-auto",
  alt = "Tekrem",
}: BrandLogoProps) {
  return (
    <span className={cn("inline-flex items-center", className)}>
      <img
        src={logoBlue}
        className={cn("block object-contain dark:hidden", imgClassName)}
        alt={alt}
      />
      <img
        src={logoLight}
        className={cn("hidden object-contain dark:block", imgClassName)}
        alt={alt}
      />
    </span>
  );
}

/** Compact mark for collapsed sidebar. */
export function BrandMark({ className, alt = "Tekrem" }: { className?: string; alt?: string }) {
  return <img src={favicon} alt={alt} className={cn("h-8 w-8 object-contain", className)} />;
}
