"use strict";

import fetchAsSets from "./lib/fetchAsSets";
import extractASNs from "./lib/extractASNs";
import generatePrefixLists, { generatePrefixListCommands } from "./lib/generatePrefixLists";
import { execSync } from "child_process";

async function main() {
  let asns = extractASNs();

  for (const asn of asns) {
    let asSets = await fetchAsSets(asn);
    let prefixLists = generatePrefixLists(`${asn}`, asSets);

    // Generate all commands for this ASN as a batch
    let commands = generatePrefixListCommands(prefixLists);

    if (commands.length > 2) { // "conf t", ...cmds..., "end"
      // Join commands with ' -c ' for vtysh batch execution
      const vtyshCmd = 'vtysh ' + commands.map(cmd => `-c "${cmd}"`).join(' ');
      execSync(vtyshCmd);
      commands.slice(1, -1).forEach(cmd => console.log(`Adding ${cmd}`));
    }
  }
}

(async () => {
  await main();
})();
