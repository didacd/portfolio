import mdx from '@astrojs/mdx';
import { unified } from '@astrojs/markdown-remark';
import sitemap from '@astrojs/sitemap';
import { defineConfig } from 'astro/config';
import expressiveCode from 'astro-expressive-code';
import { loadEnv } from 'vite';
import { readFileSync } from 'node:fs';
import { load } from 'js-yaml';
import spectre from './package/src';
import { getCalloutDefinitions, rehypeCallouts } from './src/lib/callouts';
import { spectreDark } from './src/ec-theme';

const {
	GISCUS_REPO,
	GISCUS_REPO_ID,
	GISCUS_CATEGORY,
	GISCUS_CATEGORY_ID,
	GISCUS_MAPPING,
	GISCUS_STRICT,
	GISCUS_REACTIONS_ENABLED,
	GISCUS_EMIT_METADATA,
	GISCUS_LANG,
} = loadEnv(process.env.NODE_ENV!, process.cwd(), '');

const siteConfig = load(readFileSync(new URL('./src/config/site.yaml', import.meta.url), 'utf8'));
const callouts = getCalloutDefinitions(siteConfig);

// https://astro.build/config
const config = defineConfig({
	site: 'https://didac.domenech.dev',
	output: 'static',
	markdown: {
		processor: unified({ rehypePlugins: [[rehypeCallouts, callouts]] }),
	},
	image: {
		remotePatterns: [{ protocol: 'https', hostname: 'avatars.githubusercontent.com' }],
	},
	integrations: [
		expressiveCode({
			themes: [spectreDark],
			styleOverrides: {
				codeFontFamily: "'Cascadia Code', 'Courier New', Courier, monospace",
				uiFontFamily: "'Cascadia Code', 'Courier New', Courier, monospace",
			},
		}),
		mdx(),
		sitemap(),
		spectre({
			name: 'Arcanum',
			openGraph: {
				home: {
					title: 'Home',
					description: 'My personal space.',
				},
				blog: {
					title: 'Blog',
					description: 'Sharing my inspiration.',
				},
				projects: {
					title: 'Projects',
				},
			},
			/* giscus: {
				repository: GISCUS_REPO,
				repositoryId: GISCUS_REPO_ID,
				category: GISCUS_CATEGORY,
				categoryId: GISCUS_CATEGORY_ID,
				mapping: GISCUS_MAPPING as GiscusMapping,
				strict: GISCUS_STRICT === 'true',
				reactionsEnabled: GISCUS_REACTIONS_ENABLED === 'true',
				emitMetadata: GISCUS_EMIT_METADATA === 'true',
				lang: GISCUS_LANG,
			}, */
		}),
	],
});

export default config;
