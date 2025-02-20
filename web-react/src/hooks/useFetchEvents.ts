import { getTxEvents } from "@/api/node/getTxEvents";
import { shortenString } from "@/utils/shortenString";

export const useFetchEvents = (
  onEvent: (e: string) => void,
  onSettle: () => void,
) => {
  const fetchEvents = async (tx: string, event_count: number) => {
    let events = await getTxEvents(tx as string);
    events = events.sort((a: any, b: any) => a.block_height - b.block_height);

    let count = 0;
    for (const event of events) {
      for (const e of event.events) {
        count++;
        console.log(count, event_count);
        if (count <= event_count) {
          continue;
        }

        if (e.name == "NewProof") {
          onEvent(
            //@ts-ignore
            `Transaction ${shortenString(tx, 18)} proof submitted for blob ${e.metadata.blob_index}.`,
          );
        }
        if (e.name == "Sequenced") {
          onEvent(
            `Transaction ${shortenString(tx, 18)} sequenced. Waiting for settlement...`,
          );
        }
        if (e.name == "SettledAsFailed") {
          onEvent(
            `⛈️  Transaction ${shortenString(tx, 18)} settled as failed.`,
          );
          onSettle();
          return;
        }
        if (e.name == "Settled") {
          onEvent(`✨ Transaction ${shortenString(tx, 18)} settled!`);
          onSettle();
          return;
        }
      }
    }
    setTimeout(() => fetchEvents(tx, count), 500);
  };

  return (tx: string) => fetchEvents(tx, 0);
};
