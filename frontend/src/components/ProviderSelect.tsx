import { Select } from "@/components/ui/select";
import { countryByIso2 } from "@/lib/catalog";
import { cn } from "@/lib/utils";

type ProviderSelectProps = {
  id?: string;
  countryIso2: string;
  value: string;
  onChange: (correspondent: string) => void;
  disabled?: boolean;
  className?: string;
  required?: boolean;
};

export function ProviderSelect({
  id,
  countryIso2,
  value,
  onChange,
  disabled,
  className,
  required,
}: ProviderSelectProps) {
  const mnos = countryByIso2(countryIso2)?.mnos ?? [];

  return (
    <Select
      id={id}
      value={value}
      disabled={disabled || mnos.length === 0}
      required={required}
      className={cn(className)}
      onChange={(e) => onChange(e.target.value)}
    >
      {mnos.length === 0 && <option value="">No providers</option>}
      {mnos.map((m) => (
        <option key={m.correspondent} value={m.correspondent}>
          {m.label}
        </option>
      ))}
    </Select>
  );
}
