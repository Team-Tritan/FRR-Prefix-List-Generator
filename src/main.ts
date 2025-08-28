"use strict";

import fetchAsSets from "./lib/fetchAsSets";
import extractASNs from "./lib/extractASNs";
import generatePrefixLists, {
  generatePrefixListCommands,
} from "./lib/generatePrefixLists";
import { spawn, execSync } from "child_process";
import {
  log,
  logInfo,
  logWarn,
  logError,
  color,
} from "./lib/logger";
import { vrfs } from "../config";

function getPeerIPs(asn: number, vrf: string): { v4: string[]; v6: string[] } {
  const v4: string[] = [];
  const v6: string[] = [];
  try {
    const output = execSync(`vtysh -c 'show bgp vrf ${vrf} su'`).toString();
    const blocks = output.split(/\n(?=BGP neighbor is )/);
    for (const block of blocks) {
      const ipMatch = block.match(/BGP neighbor is ([^\s,]+)/);
      if (!ipMatch) continue;
      const ip = ipMatch[1];
      const asMatch = block.match(/remote AS (\d+)/);
      if (!asMatch) continue;
      const remoteAsn = parseInt(asMatch[1], 10);
      if (remoteAsn !== asn) continue;
      if (ip.includes(":")) {
        if (!v6.includes(ip)) v6.push(ip);
      } else {
        if (!v4.includes(ip)) v4.push(ip);
      }
    }
  } catch (err) {
    logError("main", `Failed to get peer IPs for ASN ${asn} in VRF ${vrf}: ${err instanceof Error ? err.message : String(err)}`);
  }
  return { v4, v6 };
}

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
  for (const vrf of Object.keys(vrfs)) {
    log("main", `Processing VRF: ${vrf}...`, color.cyan);
    log("main", "Extracting ASNs...", color.cyan);

    const vrfASN = vrfs[vrf];

    if (!vrfASN) {
      logError("main", `No ASN configured for VRF ${vrf}. Skipping.`);
      continue;
    }

    const asns = extractASNs(vrf);
    logInfo("main", `Found ASNs in VRF ${vrf}: ${color.magenta}${asns.join(", ")}${color.reset}`);
    
    for (const asn of asns) {
      log("main", `Processing ASN ${asn} in VRF ${vrf} (ASN for VRF: ${vrfASN})...`, color.cyan);

      const asSets = await fetchAsSets(asn);
      logInfo("main", `AS-SETs for ASN ${asn}: ${color.magenta}${asSets.join(", ")}${color.reset}`);

      const prefixLists = await generatePrefixLists(`${asn}`, asSets);
      const commands = generatePrefixListCommands(prefixLists);

      log("main", `Generated ${commands.length - 2} prefix-list commands for ASN ${asn} in VRF ${vrf}.`, color.cyan);

      if (commands.length > 2) {
        const vtyshArgs: string[] = [];
        vtyshArgs.push("-c", "conf");
        vtyshArgs.push("-c", `router bgp ${vrfASN} vrf ${vrf}`);
        commands.slice(1, -1).forEach((cmd) => vtyshArgs.push("-c", cmd));
        vtyshArgs.push("-c", "end");

        log("main", `Executing vtysh for ASN ${asn} in VRF ${vrf} with ${commands.length - 2} commands...`, color.cyan);
        try {
          await runVtysh(vtyshArgs);
          commands.slice(1, -1).forEach((cmd) => logInfo("vtysh", `Applied: ${cmd}`));
          logInfo("main", `vtysh execution for ASN ${asn} in VRF ${vrf} completed.`);
        } catch (e) {
          logError("main", `vtysh command failed for ASN ${asn} in VRF ${vrf}: ${e instanceof Error ? e.message : String(e)}`);
          continue;
        }
      } else {
        logWarn("main", `No prefix-list commands to apply for ASN ${asn} in VRF ${vrf}.`);
      }

      const peerIPs = getPeerIPs(asn, vrf);
      const v4Count = prefixLists.v4.length;
      const v6Count = prefixLists.v6.length;

      const maxPrefixCmds: string[] = [];

      peerIPs.v4.forEach((peer) => {
        maxPrefixCmds.push(
          "-c", "conf",
          "-c", `router bgp ${vrfASN} vrf ${vrf}`,
          "-c", "address-family ipv4 unicast",
          "-c", `neighbor ${peer} maximum-prefix ${v4Count}`,
          "-c", "end"
        );
      });

      peerIPs.v6.forEach((peer) => {
        maxPrefixCmds.push(
          "-c", "conf",
          "-c", `router bgp ${vrfASN} vrf ${vrf}`,
          "-c", "address-family ipv6 unicast",
          "-c", `neighbor ${peer} maximum-prefix ${v6Count}`,
          "-c", "end"
        );
      });

      if (maxPrefixCmds.length > 0) {
        log("main", `Applying max-prefix settings for ASN ${asn} in VRF ${vrf}...`, color.cyan);
        try {
          await runVtysh(maxPrefixCmds);
          log("main", `Max-prefix configuration applied for ASN ${asn} in VRF ${vrf}.`, color.green);
        } catch (e) {
          logError("main", `Failed to apply max-prefix for ASN ${asn} in VRF ${vrf}: ${e instanceof Error ? e.message : String(e)}`);
        }
      }
    }
  }
  logInfo("main", "All VRFs and ASNs processed.");
}

(async () => {
  await main();
})();
=======
"use strict";

import fetchAsSets from "./lib/fetchAsSets";
import extractASNs from "./lib/extractASNs";
import generatePrefixLists, {
  generatePrefixListCommands,
} from "./lib/generatePrefixLists";
import { spawn, execSync } from "child_process";
import {
  log,
  logInfo,
  logWarn,
  logError,
  logMagenta,
  color,
} from "./lib/logger";

function getPeerIPs(asn: number): { v4: string[]; v6: string[] } {
  const v4: string[] = [];
  const v6: string[] = [];
  try {
    const output = execSync("vtysh -c 'show bgp neighbors'").toString();
    const blocks = output.split(/\n(?=BGP neighbor is )/);

    for (const block of blocks) {
      const ipMatch = block.match(/BGP neighbor is ([^\s,]+)/);
      if (!ipMatch) continue;

      const ip = ipMatch[1];
      const asMatch = block.match(/remote AS (\d+)/);

      if (!asMatch) continue;
      const remoteAsn = parseInt(asMatch[1], 10);
      
      if (remoteAsn !== asn) continue;

      if (ip.includes(":")) {
        if (!v6.includes(ip)) v6.push(ip);
      } else {
        if (!v4.includes(ip)) v4.push(ip);
      }
    }
  } catch (err) {
    logError("main", `Failed to get peer IPs for ASN ${asn}: ${err instanceof Error ? err.message : String(err)}`);
  }
  return { v4, v6 };
}

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

    const peerIPs = getPeerIPs(asn);
    const v4Count = prefixLists.v4.length;
    const v6Count = prefixLists.v6.length;

    if (peerIPs.v4.length > 0 && v4Count > 0) {
      for (const peer of peerIPs.v4) {
        const cmds = [
          "conf",
          "router bgp",
          "address-family ipv4 unicast",
          `neighbor ${peer} maximum-prefix ${v4Count}`,
          "end"
        ];
        log("main", `Setting IPv4 maximum-prefix for neighbor ${peer}: ${v4Count}`, color.cyan);
        try {
          await runVtysh(cmds.flatMap(cmd => ["-c", cmd]));
          log("main", `IPv4 maximum-prefix set for neighbor ${peer}: ${v4Count}`, color.green);
        } catch (e) {
          logError("main", `Failed to set IPv4 maximum-prefix for neighbor ${peer}: ${e instanceof Error ? e.message : String(e)}`);
        }
      }
    }

    if (peerIPs.v6.length > 0 && v6Count > 0) {
      for (const peer of peerIPs.v6) {
        const cmds = [
          "conf",
          "router bgp",
          "address-family ipv6 unicast",
          `neighbor ${peer} maximum-prefix ${v6Count}`,
          "end",
        ];
        log("main", `Setting IPv6 maximum-prefix for neighbor ${peer}: ${v6Count}`, color.cyan);
        try {
          await runVtysh(cmds.flatMap(cmd => ["-c", cmd]));
          log("main", `IPv6 maximum-prefix set for neighbor ${peer}: ${v6Count}`, color.green);
        } catch (e) {
          logError("main", `Failed to set IPv6 maximum-prefix for neighbor ${peer}: ${e instanceof Error ? e.message : String(e)}`);
        }
      }
    }
  }

  logInfo("main", "All ASNs processed.");
}

(async () => {
  await main();
})();
