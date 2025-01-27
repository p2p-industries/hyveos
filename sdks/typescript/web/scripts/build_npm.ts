import { build, emptyDir } from 'jsr:@deno/dnt'

await emptyDir('./npm')

await build({
  entryPoints: ['./mod.ts'],
  outDir: './npm',
  shims: {
    deno: false,
  },
  compilerOptions: {
    lib: ['ESNext', 'DOM'],
  },
  package: {
    name: '@hyveos/web',
    version: Deno.args[0],
    description: 'Transport provider for the hyveOS SDK for use on the web',
    license: 'MIT',
  },
})
