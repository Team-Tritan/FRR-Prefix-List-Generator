"use strict";

import fetchAsSets from "./lib/fetchAsSets";
import extractASNs from "./lib/extractASNs";
import generatePrefixLists, {
  generatePrefixListCommands,
} from "./lib/generatePrefixLists";
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

function runVtysh(args: string[]): Promise<void> {
  return new Promise((resolve, reject) => {
    const proc = spawn("vtysh", args, { stdio: "pipe" });

    proc.on("error", (err) => reject(err));
    proc.on("exit", (code, signal) => {
      if (code === 0) resolve();
      else reject(new Error(`vtysh exited with code ${code} signal ${signal}`));
    });
  });
}

async function main() {
  console.log(`${color.cyan}[main]${color.reset} Extracting ASNs...`);
  const asns = extractASNs();
  console.log(
    `${color.green}[main]${color.reset} Found ASNs: ${color.magenta}${asns.join(
      ", "
    )}${color.reset}`
  );

  for (const asn of asns) {
    console.log(`${color.cyan}[main]${color.reset} Processing ASN ${asn}...`);
    const asSets = await fetchAsSets(asn);
    console.log(
      `${color.green}[main]${color.reset} AS-SETs for ASN ${asn}: ${color.magenta
      }${asSets.join(", ")}${color.reset}`
    );
    const prefixLists = await generatePrefixLists(`${asn}`, asSets);

    const commands = generatePrefixListCommands(prefixLists);
    console.log(
      `${color.cyan}[main]${color.reset} Generated ${commands.length - 2
      } prefix-list commands for ASN ${asn}.`
    );

    if (commands.length > 2) {
      const vtyshArgs = commands.flatMap((cmd) => ["-c", cmd]);
      console.log(
        `${color.cyan}[main]${color.reset} Executing vtysh for ASN ${asn} with ${commands.length - 2
        } commands...`
      );
      try {
        await runVtysh(vtyshArgs);
        commands
          .slice(1, -1)
          .forEach((cmd) =>
            console.log(`${color.green}[vtysh]${color.reset} Adding ${cmd}`)
          );
        console.log(
          `${color.green}[main]${color.reset} vtysh execution for ASN ${asn} completed.`
        );
      } catch (e) {
        console.error(
          `${color.red}[main]${color.reset} vtysh command timed out or failed for ASN ${asn}:`,
          e instanceof Error ? e.message : String(e)
        );
        continue;
      }
    } else {
      console.log(
        `${color.yellow}[main]${color.reset} No prefix-list commands to apply for ASN ${asn}.`
      );
    }
  }
  console.log(`${color.green}[main]${color.reset} All ASNs processed.`);
}

(async () => {
  await main();
})();
