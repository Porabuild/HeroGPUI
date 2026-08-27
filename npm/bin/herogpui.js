#!/usr/bin/env node

import { run } from "../lib/launcher.js";

run(process.argv.slice(2)).catch((error) => {
  console.error(`herogpui: ${error.message}`);
  process.exitCode = 1;
});
