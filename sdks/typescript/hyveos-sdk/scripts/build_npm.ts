import { build, emptyDir } from '@deno/dnt'

await emptyDir('./npm')

await build({
  entryPoints: ['./mod.ts'],
  outDir: './npm',
  shims: {
    deno: true,
  },
  package: {
    name: 'hyveos-sdk',
    version: Deno.args[0],
    description: 'HyveOS SDK for the browser and Node.js',
  },
})
