import axios from "axios";

interface ASData {
  [key: string]: string;
}

interface ASResponse {
  data: ASData[];
}

async function fetchASNSets(asn: number): Promise<string[]> {
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

        console.log(`AS-SET for AS${asn}:`, asSets);
        return asSets;
      } else {
        console.log(`Error fetching AS-SETs for ASN ${asn}:`);
        return [`AS${asn}`];
      }
    })
    .catch(() => {
      console.log(`Error fetching AS-SETs for ASN ${asn}:`);
      return [`AS${asn}`];
    });
}

export default fetchASNSets;
