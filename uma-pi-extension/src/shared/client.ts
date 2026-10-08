import { execFile } from "node:child_process";
import * as path from "node:path";
import * as fs from "node:fs";

export function findUmaBinary(cwd: string): string {
  const isWindows = process.platform === "win32";
  const exeName = isWindows ? "uma-cli.exe" : "uma-cli";

  const candidates = [
    path.join(cwd, "uma", "target", "release", exeName),
    path.join(cwd, "uma", "target", "debug", exeName),
    path.join(__dirname, "..", "..", "..", "uma", "target", "release", exeName),
    path.join(__dirname, "..", "..", "..", "uma", "target", "debug", exeName),
    path.join(__dirname, "..", "..", "target", "release", exeName),
    path.join(__dirname, "..", "..", "target", "debug", exeName),
    isWindows ? "uma-cli.exe" : "uma-cli",
    "uma",
  ];

  for (const candidate of candidates) {
    if (fs.existsSync(candidate)) {
      return candidate;
    }
  }

  return isWindows ? "uma-cli.exe" : "uma-cli";
}

export function runUma(
  binPath: string,
  args: string[],
  cwd: string
): Promise<{ stdout: string; stderr: string; code: number }> {
  return new Promise((resolve, reject) => {
    execFile(binPath, args, { cwd }, (error, stdout, stderr) => {
      if (error && "code" in error && typeof error.code === "number" && error.code !== 0) {
        resolve({ stdout, stderr, code: error.code });
      } else if (error) {
        reject(error);
      } else {
        resolve({ stdout, stderr, code: 0 });
      }
    });
  });
}

export async function executeUma(
  cwd: string,
  args: string[],
  emptyFallback = "No facts found."
): Promise<{ content: Array<{ type: "text"; text: string }>; details: { output?: string; error?: string } }> {
  const binPath = findUmaBinary(cwd);
  try {
    const result = await runUma(binPath, args, cwd);
    if (result.code !== 0) {
      return {
        content: [{ type: "text", text: `UMA error:\n${result.stderr || result.stdout}` }],
        details: { error: result.stderr },
      };
    }
    return {
      content: [{ type: "text", text: result.stdout.trim() || emptyFallback }],
      details: { output: result.stdout.trim() },
    };
  } catch (err: unknown) {
    const message = err instanceof Error ? err.message : String(err);
    return {
      content: [{ type: "text", text: `Failed to execute uma: ${message}` }],
      details: { error: message },
    };
  }
}
