import Select from "@/components/ui/Select";

interface TokenSelectorProps {
  token: string;
  onTokenChange: (token: string) => void;
  name?: string;
}

export default function TokenSelector({
  token,
  onTokenChange,
  name = "token",
}: TokenSelectorProps) {
  return (
    <Select
      labelText="Select a token:"
      hintText={`Selected token: ${token}`}
      value={token}
      name={name}
      onChange={(e) => onTokenChange(e.target.value)}
    >
      <option value="hyllar">Hyllar</option>
      <option value="hyllar2">Hyllar2</option>
    </Select>
  );
}
