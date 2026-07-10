import assert from "node:assert/strict";
import fs from "node:fs/promises";
import path from "node:path";
import test from "node:test";

const appsRoot = path.resolve(import.meta.dirname, "..");

test("sidebar collapse avoids transitions that continuously reflow the active page", async () => {
  const source = await fs.readFile(
    path.join(appsRoot, "src", "components", "layout", "sidebar.tsx"),
    "utf8",
  );

  assert.match(source, /isSidebarOpen \? "w-60" : "w-16"/);
  assert.doesNotMatch(source, /transition-\[width\]/);
  assert.doesNotMatch(source, /transition-all/);
});

test("page fallback stays aligned with both sidebar widths", async () => {
  const source = await fs.readFile(
    path.join(
      appsRoot,
      "src",
      "components",
      "layout",
      "page-keep-alive-viewport.tsx",
    ),
    "utf8",
  );

  assert.match(source, /isSidebarOpen \? "left-60" : "left-16"/);
});
