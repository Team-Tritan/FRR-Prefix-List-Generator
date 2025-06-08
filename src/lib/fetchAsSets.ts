import axios from "axios";

const color = {
  reset: "\x1b[0m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  red: "\x1b[31m",
  cyan: "\x1b[36m",
  magenta: "\x1b[35m",
  gray: "\x1b[90m",
};

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
  console.log(
    `${color.cyan}[fetchAsSets]${color.reset} Fetching AS-SETs for ASN ${asn}...`
  );

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

        console.log(
          `${color.green}[fetchAsSets]${color.reset} AS-SET for AS${asn}: ${color.magenta}${asSets.join(
            ", "
          )}${color.reset}`
        );

        return asSets;
      } else {
        console.log(
          `${color.yellow}[fetchAsSets]${color.reset} No AS-SETs found for ASN ${asn}, using AS${asn}.`
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

        console.warn(
          `${color.yellow}[fetchAsSets]${color.reset} Rate limited by PeeringDB (HTTP 429). Waiting ${retryAfter} seconds before retrying...`
        );

        await sleep(retryAfter * 1000);
        continue;
      }

      const errorMessage = (err instanceof Error) ? err.message : String(err);

      console.log(
        `${color.red}[fetchAsSets]${color.reset} Error fetching AS-SETs for ASN ${asn}: ${errorMessage}`
      );

      return [`AS${asn}`];
    }
  }
}

export default fetchASNSets;
