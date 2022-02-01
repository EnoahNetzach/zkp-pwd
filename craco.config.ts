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
          crateDirectory: path.resolve(__dirname, 'fe-lib'),
          extraArgs: `--target web ${process.env.NODE_ENV === 'production' ? '--release' : ''}`,
          outDir: path.resolve(__dirname, 'node_modules', 'pwd-dl-zkp-fe-lib'),
        }),
      ],
    },
  },
}

export default config

/*
{
  target: [ 'browserslist' ],
  mode: 'development',
  bail: false,
  devtool: 'cheap-module-source-map',
  entry: '/Users/fabrizio.castellarin/Develop/pwd-dl-zkp/src/index.tsx',
  output: {
    path: '/Users/fabrizio.castellarin/Develop/pwd-dl-zkp/build',
    pathinfo: true,
    filename: 'static/js/bundle.js',
    chunkFilename: 'static/js/[name].chunk.js',
    assetModuleFilename: 'static/media/[name].[hash][ext]',
    publicPath: '/',
    devtoolModuleFilenameTemplate: [Function]
  },
  cache: {
    type: 'filesystem',
    version: '56e655ab29e15739b04e5da711108e4f',
    cacheDirectory: '/Users/fabrizio.castellarin/Develop/pwd-dl-zkp/node_modules/.cache',
    store: 'pack',
    buildDependencies: { defaultWebpack: [Array], config: [Array], tsconfig: [Array] }
  },
  infrastructureLogging: { level: 'none' },
  optimization: {
    minimize: false,
    minimizer: [ [TerserPlugin], [CssMinimizerPlugin] ]
  },
  resolve: {
    modules: [
      'node_modules',
      '/Users/fabrizio.castellarin/Develop/pwd-dl-zkp/node_modules'
    ],
    extensions: [
      '.web.mjs', '.mjs',
      '.web.js',  '.js',
      '.web.ts',  '.ts',
      '.web.tsx', '.tsx',
      '.json',    '.web.jsx',
      '.jsx'
    ],
    alias: { 'react-native': 'react-native-web' },
    plugins: [ [ModuleScopePlugin] ]
  },
  module: { strictExportPresence: true, rules: [ [Object], [Object] ] },
  plugins: [
    WasmPackPlugin {
      _ranInitialCompilation: false,
      crateDirectory: '/Users/fabrizio.castellarin/Develop/pwd-dl-zkp/fe-lib',
      forceWatch: undefined,
      forceMode: undefined,
      args: [Array],
      extraArgs: [],
      outDir: 'pkg',
      outName: 'index',
      watchDirectories: [Array],
      watchFiles: [Array],
      wp: [Watchpack],
      isDebug: true,
      error: null
    },
    HtmlWebpackPlugin { userOptions: [Object], version: 5 },
    InterpolateHtmlPlugin {
      htmlWebpackPlugin: [Function],
      replacements: [Object]
    },
    ModuleNotFoundPlugin {
      appPath: '/Users/fabrizio.castellarin/Develop/pwd-dl-zkp',
      yarnLockFile: undefined,
      useYarnCommand: [Function: bound useYarnCommand],
      getRelativePath: [Function: bound getRelativePath],
      prettierError: [Function: bound prettierError]
    },
    DefinePlugin { definitions: [Object] },
    ReactRefreshPlugin { options: [Object] },
    CaseSensitivePathsPlugin {
      options: {},
      logger: [Object],
      pathCache: Map {},
      fsOperations: 0,
      primed: false
    },
    WebpackManifestPlugin { options: [Object] },
    IgnorePlugin {
      options: [Object],
      checkIgnore: [Function: bound checkIgnore]
    },
    ForkTsCheckerWebpackPlugin { options: [Object] },
    ESLintWebpackPlugin {
      key: 'ESLintWebpackPlugin',
      options: [Object],
      run: [Function: bound run] AsyncFunction
    }
  ],
  performance: false
}
 */
