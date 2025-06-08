"use strict";

import fetchAsSets from "./lib/fetchAsSets";
import extractASNs from "./lib/extractASNs";
import generatePrefixLists, {
  generatePrefixListCommands,
} from "./lib/generatePrefixLists";
import { execSync } from "child_process";

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

async function main() {
  console.log(`${color.cyan}[main] Extracting ASNs...${color.reset}`);
  let asns = extractASNs();
  console.log(
    `${color.green}[main] Found ASNs:${color.reset} ${color.magenta}${asns.join(
      ", "
    )}${color.reset}`
  );

  for (const asn of asns) {
    console.log(`${color.cyan}[main] Processing ASN ${asn}...${color.reset}`);
    let asSets = await fetchAsSets(asn);
    console.log(
      `${color.green}[main] AS-SETs for ASN ${asn}:${color.reset} ${color.magenta}${asSets.join(
        ", "
      )}${color.reset}`
    );
    let prefixLists = generatePrefixLists(`${asn}`, asSets);

    let commands = generatePrefixListCommands(prefixLists);
    console.log(
      `${color.cyan}[main] Generated ${commands.length - 2} prefix-list commands for ASN ${asn}.${color.reset}`
    );

    if (commands.length > 2) {
      // "conf t", ...cmds..., "end"
      const vtyshCmd =
        "vtysh " + commands.map((cmd) => `-c "${cmd}"`).join(" ");
      console.log(
        `${color.cyan}[main] Executing vtysh for ASN ${asn} with ${commands.length - 2} commands...${color.reset}`
      );
      try {
        execSync(vtyshCmd, { timeout: 10000 }); // 10 seconds timeout
        commands.slice(1, -1).forEach((cmd) =>
          console.log(`${color.green}[vtysh] Adding ${cmd}${color.reset}`)
        );
        console.log(
          `${color.green}[main] vtysh execution for ASN ${asn} completed.${color.reset}`
        );
      } catch (e) {
        console.error(
          `${color.red}vtysh command timed out or failed for ASN ${asn}:${color.reset}`,
          (e instanceof Error ? e.message : String(e))
        );
        continue;
      }
    } else {
      console.log(
        `${color.yellow}[main] No prefix-list commands to apply for ASN ${asn}.${color.reset}`
      );
    }
  }
  console.log(`${color.green}[main] All ASNs processed.${color.reset}`);
}

(async () => {
  await main();
})();
