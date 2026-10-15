// Runs Astro's dev server for the site in `argv[2]` on port `argv[3]`, prints
// `READY <port>` when it is up, and stops on SIGTERM. A separate process, not
// Astro's `dev()` inside the test runner, whose module loader Astro's dev
// server doesn't get along with. `E2E_HOST` overrides the address it listens on.
import process from "node:process";
import { dev } from "astro";

/** Vite's base path, where its HMR socket listens. */
let hmrBase;
const server = await dev({
  root: process.argv[2],
  logLevel: "warn",
  server: { host: process.env.E2E_HOST ?? "127.0.0.1", port: Number(process.argv[3]) },
  vite: {
    plugins: [
      {
        name: "e2e-dev-server",
        configureServer(viteServer) {
          hmrBase = viteServer.config.base;
        },
      },
    ],
  },
});
await takeHeldReload(server.address.port);
process.stdout.write(`READY ${server.address.port}\n`);
process.on("SIGTERM", () => {
  server.stop().then(
    () => process.exit(0),
    () => process.exit(1),
  );
});

/**
 * Connects to the dev server's HMR socket once, and leaves. Astro's first
 * content sync asks for a full reload before any page is open; Vite holds
 * that message and sends it to the first page that connects, which reloads
 * just after it loads. A test that goes on to its next navigation in that
 * moment sees it aborted (`net::ERR_ABORTED`), as on slower runners. Taking
 * the held message here leaves the tests' pages nothing to reload for.
 */
async function takeHeldReload(port) {
  if (hmrBase === undefined) throw new Error("the e2e-dev-server plugin didn't see the dev server");
  const socket = new WebSocket(`ws://127.0.0.1:${port}${hmrBase}`, "vite-hmr");
  await new Promise((resolve, reject) => {
    socket.addEventListener("message", resolve, { once: true });
    socket.addEventListener("error", () => reject(new Error("the HMR socket failed")), {
      once: true,
    });
  });
  socket.close();
}
