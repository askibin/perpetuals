import { sveltekit } from '@sveltejs/kit/vite';
import { NodeGlobalsPolyfillPlugin } from '@esbuild-plugins/node-globals-polyfill';
import inject from '@rollup/plugin-inject';
import nodePolyfills from 'rollup-plugin-node-polyfills';
import path from 'path';

/** @type {import('vite').UserConfig} */
const config = {
	plugins: [sveltekit()],
	optimizeDeps: {
		include: ['@project-serum/anchor', '@solana/web3.js'],
		esbuildOptions: {
			target: 'esnext',
			plugins: [NodeGlobalsPolyfillPlugin({ buffer: true })]
		}
	},
	resolve: {
		alias: {
			$utils: path.resolve('src/utils/'),
			stream: 'rollup-plugin-node-polyfills/polyfills/stream'
		}
	},
	define: {
		// This makes @project-serum/anchor 's process error not happen since it replaces all instances of process.env.BROWSER with true
		'process.env.BROWSER': true,
		'process.env.NODE_DEBUG': JSON.stringify('')
	},
	build: {
		target: 'esnext',
		commonjsOptions: {
			transformMixedEsModules: true
		},
		rollupOptions: {
			plugins: [inject({ Buffer: ['buffer', 'Buffer'] }), nodePolyfills({ crypto: true })]
		}
	}
};

export default config;
