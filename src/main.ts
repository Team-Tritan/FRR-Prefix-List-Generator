"use strict";

import fetchAsSets from "./lib/fetchAsSets";
import extractASNs from "./lib/extractASNs";
import generatePrefixLists, {
  generatePrefixListCommands,
} from "./lib/generatePrefixLists";
import { spawn } from "child_process";
import {
  log,
  logInfo,
  logWarn,
  logError,
  logMagenta,
  color,
} from "./lib/logger";

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
  log("main", "Extracting ASNs...", color.cyan);

  const asns = extractASNs();
  logInfo("main", `Found ASNs: ${color.magenta}${asns.join(", ")}${color.reset}`);

  for (const asn of asns) {
    log("main", `Processing ASN ${asn}...`, color.cyan);

    const asSets = await fetchAsSets(asn);
    logInfo(
      "main",
      `AS-SETs for ASN ${asn}: ${color.magenta}${asSets.join(", ")}${color.reset}`
    );

    const prefixLists = await generatePrefixLists(`${asn}`, asSets);

    const commands = generatePrefixListCommands(prefixLists);
    log(
      "main",
      `Generated ${commands.length - 2} prefix-list commands for ASN ${asn}.`,
      color.cyan
    );

    if (commands.length > 2) {
      const vtyshArgs = commands.flatMap((cmd) => ["-c", cmd]);

      log(
        "main",
        `Executing vtysh for ASN ${asn} with ${commands.length - 2} commands...`,
        color.cyan
      );

      try {
        await runVtysh(vtyshArgs);
        commands
          .slice(1, -1)
          .forEach((cmd) => logInfo("vtysh", `Adding ${cmd}`));

        logInfo("main", `vtysh execution for ASN ${asn} completed.`);
      } catch (e) {
        logError(
          "main",
          `vtysh command timed out or failed for ASN ${asn}: ${e instanceof Error ? e.message : String(e)
          }`
        );

        continue;
      }
    } else {
      logWarn("main", `No prefix-list commands to apply for ASN ${asn}.`);
    }
  }

  logInfo("main", "All ASNs processed.");
}

(async () => {
  await main();
})();
