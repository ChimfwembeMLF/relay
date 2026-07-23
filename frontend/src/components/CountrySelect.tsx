import { Select } from "@/components/ui/select";
import { countryByIso2, type CountryEntry } from "@/lib/catalog";
import { cn } from "@/lib/utils";

type CountrySelectProps = {
  id?: string;
  value: string;
  onChange: (iso2: string) => void;
  options: CountryEntry[];
  disabled?: boolean;
  className?: string;
  required?: boolean;
};

export function CountrySelect({
  id,
  value,
  onChange,
  options,
  disabled,
  className,
  required,
}: CountrySelectProps) {
  return (
    <Select
      id={id}
      value={value}
      disabled={disabled || options.length === 0}
      required={required}
      className={cn(className)}
      onChange={(e) => onChange(e.target.value)}
    >
      {options.length === 0 && <option value="">No countries enabled</option>}
      {options.map((c) => (
        <option key={c.iso2} value={c.iso2}>
          {c.flag} {c.name} ({c.iso2})
        </option>
      ))}
    </Select>
  );
}

export function countryLabel(iso2: string): string {
  const c = countryByIso2(iso2);
  return c ? `${c.flag} ${c.name}` : iso2;
}
