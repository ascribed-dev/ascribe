import * as path from "node:path";
import Mocha from "mocha";

declare const __dirname: string;

/** The extension host calls this to run the suite named by TESSERA_SUITE. */
export async function run(): Promise<void> {
  const suite = process.env["TESSERA_SUITE"];
  if (!suite) throw new Error("TESSERA_SUITE isn't set");
  const mocha = new Mocha({ ui: "bdd", color: true, timeout: 60_000 });
  mocha.addFile(path.join(__dirname, `${suite}.it.cjs`));
  await new Promise<void>((resolve, reject) => {
    mocha.run((failures) =>
      failures > 0 ? reject(new Error(`${failures} test(s) failed`)) : resolve(),
    );
  });
}
