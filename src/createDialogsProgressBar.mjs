import * as cliProgress from "cli-progress";

export function createDialogsProgressBar(operation) {
  return new cliProgress.SingleBar(
    {
      format: `${operation} [{bar}] {percentage}% | ETA: {eta}s | {value}/{total} | {dialog}`,
    },
    cliProgress.Presets.shades_classic
  );
}
