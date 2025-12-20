#!/usr/bin/env -S deno run --allow-all
import $ from "jsr:@david/dax"; // "dax-sh" in Node
// Or `deno add jsr:@david/dax` and `import "@david/dax"`.

// run a command
await $`echo 5`; // outputs: 5

// outputting to stdout and running a sub process
await $`echo 1 && deno run main.ts`;

// parallel
await Promise.all([
  $`sleep 1 ; echo 1`,
  $`sleep 2 ; echo 2`,
  $`sleep 3 ; echo 3`,
]);
