import axios from "axios";
import { log, logInfo, logWarn, logError, logMagenta, color } from "./logger";

interface ASData {
  [key: string]: string;
}

interface ASResponse {
  data: ASData[];
}

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function fetchASNSets(asn: number): Promise<string[]> {
  log("fetchAsSets", `Fetching AS-SETs for ASN ${asn}...`, color.cyan);

  let attempt = 0;
  while (true) {
    attempt++;
    try {
      const response = await axios.get<ASResponse>(
        `https://www.peeringdb.com/api/as_set/${asn}`
      );

      if (response.data.data && response.data.data.length > 0) {
        const asSets: string[] = [];

        for (const item of response.data.data) {
          for (const asSet of Object.values(item)) {
            if (asSet !== "" && !asSets.includes(asSet)) asSets.push(asSet);
          }
        }

        logInfo(
          "fetchAsSets",
          `AS-SET for AS${asn}: ${color.magenta}${asSets.join(
            ", "
          )}${color.reset}`
        );
        return asSets;
      } else {
        logWarn(
          "fetchAsSets",
          `No AS-SETs found for ASN ${asn}, using AS${asn}.`
        );
        return [`AS${asn}`];
      }
    } catch (err: any) {
      if (err.response && err.response.status === 429) {
        let retryAfter = 60 * 3;
        const header = err.response.headers["retry-after"];
        if (header) {
          const parsed = parseInt(header, 10);
          if (!isNaN(parsed)) retryAfter = parsed;
        }
        logWarn(
          "fetchAsSets",
          `Rate limited by PeeringDB (HTTP 429). Waiting ${retryAfter} seconds before retrying...`
        );
        await sleep(retryAfter * 1000);
        continue;
      }

      const errorMessage = err instanceof Error ? err.message : String(err);
      logError("fetchAsSets", `Error fetching AS-SETs for ASN ${asn}: ${errorMessage}`);
      return [`AS${asn}`];
    }
  }
}

export default fetchASNSets;
