"use strict";

import fetchAsSets from "./lib/fetchAsSets";
import extractASNs from "./lib/extractASNs";
import generatePrefixLists, {
  generatePrefixListCommands,
} from "./lib/generatePrefixLists";
import { execSync } from "child_process";

async function main() {
  let asns = extractASNs();

  for (const asn of asns) {
    let asSets = await fetchAsSets(asn);
    let prefixLists = generatePrefixLists(`${asn}`, asSets);

    let commands = generatePrefixListCommands(prefixLists);

    if (commands.length > 2) {
      // "conf t", ...cmds..., "end"
      const vtyshCmd =
        "vtysh " + commands.map((cmd) => `-c "${cmd}"`).join(" ");
      try {
        execSync(vtyshCmd, { timeout: 10000 }); // 10 seconds timeout
        commands.slice(1, -1).forEach((cmd) => console.log(`Adding ${cmd}`));
      } catch (e) {
        console.error(
          `vtysh command timed out or failed for ASN ${asn}:`,
          (e instanceof Error ? e.message : String(e))
        );
        continue;
      }
    }
  }
}

(async () => {
  await main();
})();
