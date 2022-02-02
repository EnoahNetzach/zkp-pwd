import { CracoConfig } from '@craco/craco'
import WasmPackPlugin from '@wasm-tool/wasm-pack-plugin'
import path from 'path'

const config: CracoConfig = {
  webpack: {
    configure: (webpackConfig) => ({
      ...webpackConfig,
      experiments: {
        asyncWebAssembly: true,
        syncWebAssembly: true,
      },
      resolve: {
        ...webpackConfig.resolve,
        extensions: [...(webpackConfig.resolve?.extensions ?? []), '.wasm'],
      },
      snapshot: {
        ...webpackConfig.snapshot,
        managedPaths: [/\.\/node_modules\/(?!pwd-dl-zkp-fe-lib).*/gm],
      },
    }),
    plugins: {
      add: [
        new WasmPackPlugin({
          crateDirectory: path.resolve(__dirname, 'lib', 'frontend'),
          extraArgs: `--target web ${process.env.NODE_ENV === 'production' ? '--release' : ''}`,
          outDir: path.resolve(__dirname, 'node_modules', 'pwd-dl-zkp-fe-lib'),
        }),
      ],
    },
  },
}

export default config
