import { build, emptyDir } from "@deno/dnt";

await emptyDir("./npm");

await build({
  entryPoints: ["./mod.ts"],
  outDir: "./npm",
  importMap: "../deno.json",
  shims: {
    deno: false,
  },
  package: {
    name: "hyveos-web",
    version: Deno.args[0],
    description: "HyveOS connector for the browser",
  },
});
