import { getTxEvents } from "@/api/node/getTxEvents";
import { shortenString } from "@/utils/shortenString";

export const useFetchEvents = (
  onEvent: (e: string) => void,
  onSettle: () => void,
) => {
  const fetchEvents = async (tx: string) => {
    const events = await getTxEvents(tx as string);
    for (const event of events) {
      for (const e of event.events) {
        console.log(e);
        if (e.name == "Settled" || e.name == "Sequenced") {
          onEvent(
            `Transaction ${shortenString(tx, 18)} sequenced. Waiting for settlement...`,
          );
        }
        if (e.name == "Settled") {
          onEvent(`✨ Transaction ${shortenString(tx, 18)} settled!`);
          onSettle();
          return;
        }
      }
    }
    setTimeout(() => fetchEvents(tx), 500);
  };

  return fetchEvents;
};
