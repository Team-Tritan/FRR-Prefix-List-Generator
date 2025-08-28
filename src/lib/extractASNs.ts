import { execSync } from "child_process";
import { ignoreList } from "../../config";
import { log, logInfo, logWarn, logError, logMagenta, color } from "./logger";

function extractASNs(vrf: string): number[] {
  const asNumbers: number[] = [];
  try {
    log("extractASNs", `Running vtysh to extract ASNs for VRF ${vrf}...`, color.cyan);

    const commandOutput = execSync(`sudo vtysh -c 'show bgp vrf ${vrf} su'`).toString();
    const lines = commandOutput.split("\n");

    for (let i = 6; i < lines.length; i++) {
      const columns = lines[i].trim().split(/\s+/);

      if (columns.length >= 3) {
        const AS = parseInt(columns[2]);
        if (!isNaN(AS) && !ignoreList.includes(AS) && !asNumbers.includes(AS))
          asNumbers.push(AS);
      }
    }

    logInfo(
      "extractASNs",
      `ASNs found in VRF ${vrf}: ${color.magenta}${asNumbers.join(", ")}${color.reset}`
    );
  } catch (error) {
    logError("extractASNs", `Error executing BGP command for VRF ${vrf}: ${error}`);
  }
  return asNumbers;
}

export default extractASNs;
