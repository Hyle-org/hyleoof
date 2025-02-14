export function shortenString(str: string, maxLength: number): string {
  if (maxLength <= 0) {
    return str;
  }
  const ellipsis = "[...]";

  if (str.length <= maxLength) {
    return str;
  }

  if (maxLength <= ellipsis.length) {
    return str.slice(0, maxLength);
  }

  const charsToShow = maxLength - ellipsis.length;
  const frontChars = Math.ceil(charsToShow / 2);
  const backChars = Math.floor(charsToShow / 2);

  return (
    str.substring(0, frontChars) +
    ellipsis +
    str.substring(str.length - backChars)
  );
}
