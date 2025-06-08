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

async function fetchASNSets(asn: number): Promise<string[]> {
  console.log(
    `${color.cyan}[fetchAsSets]${color.reset} Fetching AS-SETs for ASN ${asn}...`
  );
  return axios
    .get<ASResponse>(`https://www.peeringdb.com/api/as_set/${asn}`)
    .then((response) => {
      if (response.data.data && response.data.data.length > 0) {
        const asSets: string[] = [];

        response.data.data.forEach((item: ASData) => {
          Object.values(item).forEach((asSet) => {
            if (asSet !== "" && !asSets.includes(asSet)) asSets.push(asSet);
          });
        });

        console.log(
          `${color.green}[fetchAsSets]${color.reset} AS-SET for AS${asn}: ${
            color.magenta
          }${asSets.join(", ")}${color.reset}`
        );
        return asSets;
      } else {
        console.log(
          `${color.yellow}[fetchAsSets]${color.reset} No AS-SETs found for ASN ${asn}, using AS${asn}.`
        );
        return [`AS${asn}`];
      }
    })
    .catch((err) => {
      console.log(
        `${color.red}[fetchAsSets]${
          color.reset
        } Error fetching AS-SETs for ASN ${asn}: ${err?.message || err}`
      );
      return [`AS${asn}`];
    });
}

export default fetchASNSets;
