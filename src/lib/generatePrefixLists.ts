import { execSync } from "child_process";

interface PrefixLists {
  v4: string[];
  v6: string[];
}

function generatePrefixLists(asn: string, asSets: string[]): PrefixLists {
  const results: PrefixLists = { v4: [], v6: [] };

  if (asSets && asSets.length > 0)
    asSets.forEach((asSet: string) => {
      let namingFormatV4 = `AS${asn}-In-v4`;
      let namingFormatV6 = `AS${asn}-In-v6`;

      let bgpq4IPv4Command = `bgpq4 ${asSet} -l ${namingFormatV4} -S AFRINIC,ARIN,APNIC,LACNIC,RIPE`;
      let bgpq4IPv6Command = `bgpq4 -6 ${asSet} -l ${namingFormatV6} -S AFRINIC,ARIN,APNIC,LACNIC,RIPE`;

      let resultIPv4 = execSync(bgpq4IPv4Command, { encoding: "utf-8" });
      let resultIPv6 = execSync(bgpq4IPv6Command, { encoding: "utf-8" });

      let linesIPv4 = resultIPv4.trim().split("\n");
      let linesIPv6 = resultIPv6.trim().split("\n");

      for (let i = 0; i < linesIPv4.length; i++)
        if (!results.v4.includes(linesIPv4[i])) results.v4.push(linesIPv4[i]);

      for (let i = 0; i < linesIPv6.length; i++)
        if (!results.v6.includes(linesIPv6[i])) results.v6.push(linesIPv6[i]);
    });

  return results;
}

export function generatePrefixListCommands(prefixLists: PrefixLists): string[] {
  // Combine v4 and v6 prefix list lines, filter out lines starting with "no"
  const commands = [...prefixLists.v4, ...prefixLists.v6].filter(
    (line) => !line.startsWith("no")
  );
  // Wrap with FRR CLI commands
  return ["conf t", ...commands, "end"];
}

export default generatePrefixLists;
