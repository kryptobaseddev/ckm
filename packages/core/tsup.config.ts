import { defineConfig } from 'tsup';

export default defineConfig({
  entry: [
    'src/index.ts',
    'src/adapters/commander.ts',
    'src/adapters/citty.ts',
    'src/adapters/oclif.ts',
    'src/adapters/clipanion.ts',
  ],
  format: ['esm'],
  dts: true,
  clean: true,
  sourcemap: true,
  splitting: false,
});
