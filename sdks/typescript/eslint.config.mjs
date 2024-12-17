import globals from 'globals';
import pluginJs from '@eslint/js';
import tseslint from 'typescript-eslint';

/** @type {import('eslint').Linter.Config[]} */
export default [
  { files: ['**/*.{js,mjs,cjs,ts}'] },
  { languageOptions: { globals: { ...globals.browser, ...globals.node } } },
  { ignores: ['node_modules', '**/*/dist', 'packages/hyveos-sdk/src/gen'] },
  pluginJs.configs.recommended,
  ...tseslint.configs.recommended
];
