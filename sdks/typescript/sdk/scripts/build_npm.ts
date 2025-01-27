import { build, emptyDir } from 'jsr:@deno/dnt'

await emptyDir('./npm')

await build({
  entryPoints: ['./mod.ts'],
  outDir: './npm',
  importMap: 'deno.json',
  shims: {
    deno: false,
  },
  compilerOptions: {
    lib: ['ESNext', 'DOM'],
  },
  package: {
    name: '@hyveos/sdk',
    version: Deno.args[0],
    description: 'Core SDK for hyveOS',
    license: 'MIT',
  },
})
