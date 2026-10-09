import { describe } from "vitest";
import { ENGINES } from "./harness.js";
import { suiteFor } from "./suite.js";

describe.skipIf(!ENGINES.includes("webkit"))("webkit", () => suiteFor("webkit"));
