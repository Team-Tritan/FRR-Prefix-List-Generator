import { execSync } from "child_process";
import { ingoreList } from "../../config";

function extractASNs(): number[] {
  const asNumbers: number[] = [];

  try {
    const commandOutput = execSync("sudo vtysh -c 'sh bgp su'").toString();
    const lines = commandOutput.split("\n");

    for (let i = 6; i < lines.length; i++) {
      const columns = lines[i].trim().split(/\s+/);
      if (columns.length >= 3) {
        const AS = parseInt(columns[2]);
        if (
          !isNaN(AS) &&
          AS >= 1 &&
          AS <= 65535 &&
          !ingoreList.includes(AS) &&
          !asNumbers.includes(AS)
        ) {
          asNumbers.push(AS);
        }
      }
    }

    console.log(`${asNumbers.length} AS Numbers:`, asNumbers);
  } catch (error) {
    console.error("Error executing BGP command:", error);
  }

  return asNumbers;
}

export default extractASNs;
