import { spawnSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const script = fileURLToPath(new URL('./prepare-installer.ps1', import.meta.url));
const args = ['-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', script];
let result = spawnSync('pwsh', args, { stdio: 'inherit', windowsHide: true });
if (result.error?.code === 'ENOENT') {
  // A PowerShell 7 parent may export a module path incompatible with Windows
  // PowerShell. Let Windows PowerShell construct its own standard module path.
  const env = { ...process.env };
  for (const key of Object.keys(env)) if (key.toLowerCase() === 'psmodulepath') delete env[key];
  result = spawnSync('powershell.exe', args, { stdio: 'inherit', windowsHide: true, env });
}
if (result.error) console.error(result.error.message);
process.exit(result.status ?? 1);
