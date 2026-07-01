import { expect, test } from "bun:test";
import { toksaveAbs } from "../util/paths.js";

test("toksaveAbs returns proxy literal", () => {
  const abs = toksaveAbs();
  // We expect "toksave" instead of process.argv[0] to prevent the npm wrapper node bug
  expect(abs).toBe("toksave");
});
