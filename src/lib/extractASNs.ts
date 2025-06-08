import { execSync } from "child_process";
import { ignoreList } from "../../config";

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

function extractASNs(): number[] {
  const asNumbers: number[] = [];

  try {
    console.log(
      `${color.cyan}[extractASNs] Running vtysh to extract ASNs...${color.reset}`
    );
    const commandOutput = execSync("sudo vtysh -c 'sh bgp su'").toString();
    const lines = commandOutput.split("\n");

    for (let i = 6; i < lines.length; i++) {
      const columns = lines[i].trim().split(/\s+/);

      if (columns.length >= 3) {
        const AS = parseInt(columns[2]);

        if (!isNaN(AS) && !ignoreList.includes(AS) && !asNumbers.includes(AS))
          asNumbers.push(AS);
      }
    }

    console.log(
      `${color.green}[extractASNs] ASNs found:${color.reset} ${color.magenta}${asNumbers.join(
        ", "
      )}${color.reset}`
    );
  } catch (error) {
    console.error(
      `${color.red}[extractASNs] Error executing BGP command:${color.reset}`,
      error
    );
  }

  return asNumbers;
}

export default extractASNs;
