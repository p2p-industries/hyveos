import { build, emptyDir } from 'jsr:@deno/dnt'

await emptyDir('./npm')

await build({
  entryPoints: ['./mod.ts'],
  outDir: './npm',
  shims: {
    deno: true,
  },
  package: {
    name: '@hyveos/server',
    version: Deno.args[0],
    description: 'HyveOS connection for the server',
    license: 'MIT',
  },
})