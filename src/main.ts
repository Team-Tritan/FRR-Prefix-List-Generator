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

function runVtyshWithTimeout(args: string[], timeoutMs: number): Promise<void> {
  return new Promise((resolve, reject) => {
    const proc = spawn("vtysh", args, { stdio: "pipe" });
    let timedOut = false;

    const timeout = setTimeout(() => {
      timedOut = true;
      proc.kill("SIGKILL");
      reject(new Error("vtysh timed out"));
    }, timeoutMs);

    proc.on("error", (err) => {
      clearTimeout(timeout);
      reject(err);
    });

    proc.on("exit", (code, signal) => {
      clearTimeout(timeout);
      if (timedOut) return;
      if (code === 0) {
        resolve();
      } else {
        reject(new Error(`vtysh exited with code ${code} signal ${signal}`));
      }
    });
  });
}

async function main() {
  console.log(`${color.cyan}[main]${color.reset} Extracting ASNs...`);
  let asns = extractASNs();
  console.log(
    `${color.green}[main]${color.reset} Found ASNs: ${color.magenta}${asns.join(
      ", "
    )}${color.reset}`
  );

  for (const asn of asns) {
    console.log(`${color.cyan}[main]${color.reset} Processing ASN ${asn}...`);
    let asSets = await fetchAsSets(asn);
    console.log(
      `${color.green}[main]${color.reset} AS-SETs for ASN ${asn}: ${color.magenta}${asSets.join(
        ", "
      )}${color.reset}`
    );
    let prefixLists = await generatePrefixLists(`${asn}`, asSets);

    let commands = generatePrefixListCommands(prefixLists);
    console.log(
      `${color.cyan}[main]${color.reset} Generated ${commands.length - 2} prefix-list commands for ASN ${asn}.`
    );

    if (commands.length > 2) {
      // "conf t", ...cmds..., "end"
      const vtyshArgs = commands.map((cmd) => `-c`).flatMap((c, i) => [c, commands[i]]);
      console.log(
        `${color.cyan}[main]${color.reset} Executing vtysh for ASN ${asn} with ${commands.length - 2} commands...`
      );
      try {
        await runVtyshWithTimeout(vtyshArgs, 10000);
        commands.slice(1, -1).forEach((cmd) =>
          console.log(`${color.green}[vtysh]${color.reset} Adding ${cmd}`)
        );
        console.log(
          `${color.green}[main]${color.reset} vtysh execution for ASN ${asn} completed.`
        );
      } catch (e) {
        console.error(
          `${color.red}[main]${color.reset} vtysh command timed out or failed for ASN ${asn}:`,
          (e instanceof Error ? e.message : String(e))
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
