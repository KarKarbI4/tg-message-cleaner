import * as fs from "fs";
import * as path from "path";

const tmpDir = "tmp/";

export function writeToTmpFileSync(filename, data) {
  if (!fs.existsSync(tmpDir)) {
    fs.mkdirSync(tmpDir);
  }
  const tmpPath = path.join(tmpDir, filename);
  if (!tmpPath.startsWith(tmpDir)) {
    throw new Error(`Trying to write file not in tmp folder, path: ${tmpPath}`);
  }
  fs.writeFileSync(tmpPath, data);
}
