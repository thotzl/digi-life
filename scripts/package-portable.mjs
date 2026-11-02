// Portable Windows build (TCK-195): packs the release exe of `tauri build` and the readme into
// target/release/bundle/portable/<productName>_<version>_x64-portable.zip. Run after `tauri build` (npm run release:win).
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync } from "node:fs";
import { join } from "node:path";

const conf = JSON.parse(readFileSync("tauri.conf.json", "utf8"));
const { version } = JSON.parse(readFileSync("package.json", "utf8"));
const name = conf.mainBinaryName || conf.productName;
const exe = join("target", "release", `${name}.exe`);
if (!existsSync(exe)) {
  console.error(`[portable] ${exe} not found; run tauri build first`);
  process.exit(1);
}
const outDir = join("target", "release", "bundle", "portable");
mkdirSync(outDir, { recursive: true });
const zip = join(outDir, `${name}_${version}_x64-portable.zip`);
rmSync(zip, { force: true });
const files = [exe, join("scripts", "portable", "LIESMICH.txt")].map((f) => `'${f}'`).join(",");
execFileSync("powershell", ["-NoProfile", "-Command", `Compress-Archive -Path ${files} -DestinationPath '${zip}'`], { stdio: "inherit" });
console.log(`[portable] ${zip}`);
