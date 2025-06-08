import { spawn } from "child_process";

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

async function runBGPQ4Async(
  command: string,
  asSet: string,
  version: string,
  timeoutMs = 10000
): Promise<string> {
  return new Promise((resolve) => {
    console.log(
      `${color.cyan}[bgpq4]${color.reset} Running ${version} command for ${asSet}: ${color.gray}${command}${color.reset}`
    );

    const [cmd, ...args] = command.split(" ");
    const proc = spawn(cmd, args, { stdio: ["ignore", "pipe", "pipe"] });

    let stdout = "";
    let finished = false;

    const timeout = setTimeout(() => {
      if (!finished) {
        finished = true;
        proc.kill("SIGKILL");

        console.warn(
          `${color.yellow}[bgpq4]${color.reset} ${version} command for ${asSet} timed out.`
        );

        resolve("");
      }
    }, timeoutMs);

    proc.stdout.on("data", (data) => {
      stdout += data.toString();
    });

    proc.on("error", (err) => {
      if (!finished) {
        finished = true;
        clearTimeout(timeout);

        console.warn(
          `${color.red}[bgpq4]${color.reset} ${version} command for ${asSet} failed: ${err.message}`
        );

        resolve("");
      }
    });

    proc.on("exit", (code) => {
      if (!finished) {
        finished = true;

        clearTimeout(timeout);

        if (code === 0) {
          console.log(
            `${color.green}[bgpq4]${color.reset} ${version} command for ${asSet} completed successfully.`
          );

          resolve(stdout);
        } else {
          console.warn(
            `${color.red}[bgpq4]${color.reset} ${version} command for ${asSet} exited with code ${code}.`
          );

          resolve("");
        }
      }
    });
  });
}

export async function generatePrefixLists(
  asn: string,
  asSets: string[]
): Promise<PrefixLists> {
  const results: PrefixLists = { v4: [], v6: [] };

  if (asSets && asSets.length > 0) {
    for (const asSet of asSets) {
      const namingFormatV4 = `AS${asn}-In-v4`;
      const namingFormatV6 = `AS${asn}-In-v6`;

      const bgpq4IPv4Command = `bgpq4 ${asSet} -l ${namingFormatV4} -S AFRINIC,ARIN,APNIC,LACNIC,RIPE`;
      const bgpq4IPv6Command = `bgpq4 -6 ${asSet} -l ${namingFormatV6} -S AFRINIC,ARIN,APNIC,LACNIC,RIPE`;

      const resultIPv4 = await runBGPQ4Async(bgpq4IPv4Command, asSet, "IPv4");
      const resultIPv6 = await runBGPQ4Async(bgpq4IPv6Command, asSet, "IPv6");

      const linesIPv4 = resultIPv4.trim() ? resultIPv4.trim().split("\n") : [];
      const linesIPv6 = resultIPv6.trim() ? resultIPv6.trim().split("\n") : [];

      if (linesIPv4.length > 0)
        console.log(
          `${color.magenta}[bgpq4]${color.reset} Parsed ${linesIPv4.length} IPv4 prefix-list lines for ${asSet}.`
        );

      if (linesIPv6.length > 0)
        console.log(
          `${color.magenta}[bgpq4]${color.reset} Parsed ${linesIPv6.length} IPv6 prefix-list lines for ${asSet}.`
        );

      for (const line of linesIPv4)
        if (!results.v4.includes(line)) results.v4.push(line);

      for (const line of linesIPv6)
        if (!results.v6.includes(line)) results.v6.push(line);
    }
  }

  return results;
}

export function generatePrefixListCommands(prefixLists: PrefixLists): string[] {
  const commands = [...prefixLists.v4, ...prefixLists.v6].filter(
    (line) => !line.startsWith("no")
  );
  
  return ["conf t", ...commands, "end"];
}

export default generatePrefixLists;
