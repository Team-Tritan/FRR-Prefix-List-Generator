"use strict";

import fetchAsSets from "./lib/fetchAsSets";
import extractASNs from "./lib/extractASNs";
import generatePrefixLists, {
  generatePrefixListCommands,
} from "./lib/generatePrefixLists";
import { execSync } from "child_process";

async function main() {
  console.log("[main] Extracting ASNs...");
  let asns = extractASNs();
  console.log(`[main] Found ASNs: ${asns.join(", ")}`);

  for (const asn of asns) {
    console.log(`[main] Processing ASN ${asn}...`);
    let asSets = await fetchAsSets(asn);
    console.log(`[main] AS-SETs for ASN ${asn}: ${asSets.join(", ")}`);
    let prefixLists = generatePrefixLists(`${asn}`, asSets);

    let commands = generatePrefixListCommands(prefixLists);
    console.log(
      `[main] Generated ${commands.length - 2} prefix-list commands for ASN ${asn}.`
    );

    if (commands.length > 2) {
      // "conf t", ...cmds..., "end"
      const vtyshCmd =
        "vtysh " + commands.map((cmd) => `-c "${cmd}"`).join(" ");
      console.log(
        `[main] Executing vtysh for ASN ${asn} with ${commands.length - 2} commands...`
      );
      try {
        execSync(vtyshCmd, { timeout: 10000 }); // 10 seconds timeout
        commands.slice(1, -1).forEach((cmd) =>
          console.log(`[vtysh] Adding ${cmd}`)
        );
        console.log(`[main] vtysh execution for ASN ${asn} completed.`);
      } catch (e) {
        console.error(
          `vtysh command timed out or failed for ASN ${asn}:`,
          (e instanceof Error ? e.message : String(e))
        );
        continue;
      }
    } else {
      console.log(`[main] No prefix-list commands to apply for ASN ${asn}.`);
    }
  }
  console.log("[main] All ASNs processed.");
}

(async () => {
  await main();
})();
