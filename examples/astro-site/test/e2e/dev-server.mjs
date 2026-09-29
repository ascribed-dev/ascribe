// Runs Astro's dev server for the site in `argv[2]` on port `argv[3]`, prints
// `READY <port>` when it is up, and stops on SIGTERM. A separate process, not
// Astro's `dev()` inside the test runner, whose module loader Astro's dev
// server doesn't get along with.
import process from "node:process";
import { dev } from "astro";

const server = await dev({ root: process.argv[2], logLevel: "warn", server: { host: "127.0.0.1", port: Number(process.argv[3]) } });
process.stdout.write(`READY ${server.address.port}\n`);
process.on("SIGTERM", () => {
  server.stop().then(() => process.exit(0));
});
