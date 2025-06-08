import { spawnSync } from "child_process";

// Color helpers
const color = {
  reset: "\x1b[0m",
  green: "\x1b[32m",
  yellow: "\x1b[33m",
  red: "\x1b[31m",
  cyan: "\x1b[36m",
  magenta: "\x1b[35m",
  gray: "\x1b[90m",
};

interface PrefixLists {
  v4: string[];
  v6: string[];
}

function runBGPQ4(command: string, asSet: string, version: string): string {
  console.log(
    `${color.cyan}[bgpq4] Running ${version} command for ${asSet}:${color.reset} ${color.gray}${command}${color.reset}`
  );
  const [cmd, ...args] = command.split(" ");
  const result = spawnSync(cmd, args, {
    encoding: "utf-8",
    timeout: 10000,
    maxBuffer: 10 * 1024 * 1024, // 10MB buffer for large outputs
  });
  if (result.error) {
    if ((result.error as any).code === "ETIMEDOUT") {
      console.warn(
        `${color.yellow}[bgpq4] ${version} command for ${asSet} timed out.${color.reset}`
      );
    } else {
      console.warn(
        `${color.red}[bgpq4] ${version} command for ${asSet} failed: ${result.error.message}${color.reset}`
      );
    }
    return "";
  }
  if (result.status !== 0) {
    console.warn(
      `${color.red}[bgpq4] ${version} command for ${asSet} exited with code ${result.status}.${color.reset}`
    );
    return "";
  }
  console.log(
    `${color.green}[bgpq4] ${version} command for ${asSet} completed successfully.${color.reset}`
  );
  return result.stdout;
}

function generatePrefixLists(asn: string, asSets: string[]): PrefixLists {
  const results: PrefixLists = { v4: [], v6: [] };

  if (asSets && asSets.length > 0)
    asSets.forEach((asSet: string) => {
      let namingFormatV4 = `AS${asn}-In-v4`;
      let namingFormatV6 = `AS${asn}-In-v6`;

      let bgpq4IPv4Command = `bgpq4 ${asSet} -l ${namingFormatV4} -S AFRINIC,ARIN,APNIC,LACNIC,RIPE`;
      let bgpq4IPv6Command = `bgpq4 -6 ${asSet} -l ${namingFormatV6} -S AFRINIC,ARIN,APNIC,LACNIC,RIPE`;

      let resultIPv4 = runBGPQ4(bgpq4IPv4Command, asSet, "IPv4");
      let resultIPv6 = runBGPQ4(bgpq4IPv6Command, asSet, "IPv6");

      let linesIPv4 = resultIPv4.trim() ? resultIPv4.trim().split("\n") : [];
      let linesIPv6 = resultIPv6.trim() ? resultIPv6.trim().split("\n") : [];

      if (linesIPv4.length > 0) {
        console.log(
          `${color.magenta}[bgpq4] Parsed ${linesIPv4.length} IPv4 prefix-list lines for ${asSet}.${color.reset}`
        );
      }
      if (linesIPv6.length > 0) {
        console.log(
          `${color.magenta}[bgpq4] Parsed ${linesIPv6.length} IPv6 prefix-list lines for ${asSet}.${color.reset}`
        );
      }

      for (let i = 0; i < linesIPv4.length; i++)
        if (!results.v4.includes(linesIPv4[i])) results.v4.push(linesIPv4[i]);

      for (let i = 0; i < linesIPv6.length; i++)
        if (!results.v6.includes(linesIPv6[i])) results.v6.push(linesIPv6[i]);
    });

  return results;
}

export function generatePrefixListCommands(prefixLists: PrefixLists): string[] {
  const commands = [...prefixLists.v4, ...prefixLists.v6].filter(
    (line) => !line.startsWith("no")
  );
  return ["conf t", ...commands, "end"];
}

export default generatePrefixLists;
