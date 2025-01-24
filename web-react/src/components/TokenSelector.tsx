import Select from "@/components/ui/Select";

interface TokenSelectorProps {
  token: string;
  onTokenChange: (token: string) => void;
}

export default function TokenSelector({
  token,
  onTokenChange,
}: TokenSelectorProps) {
  return (
    <Select
      labelText="Select a token:"
      hintText={`Selected token: ${token}`}
      value={token}
      onChange={(e) => onTokenChange(e.target.value)}
    >
      <option value="hyllar">Hyllar</option>
      <option value="hyllar2">Hyllar2</option>
    </Select>
  );
}
