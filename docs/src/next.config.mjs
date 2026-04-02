import { createMDX } from 'fumadocs-mdx/next';

const withMDX = createMDX();

// CDN origin for binary assets (WASM, KB). In production, Vercel rewrites
// proxy these paths to GitHub Releases so the browser sees same-origin
// requests with correct Content-Type headers.
const CDN_ORIGIN =
  process.env.CDN_ORIGIN ||
  'https://github.com/weave-logic-ai/weftos/releases/download/cdn-assets';

/** @type {import('next').NextConfig} */
const config = {
  reactStrictMode: true,
  turbopack: {
    root: import.meta.dirname,
  },
  async rewrites() {
    // Only rewrite when the files don't exist locally (production).
    // In local dev, public/wasm/ and public/kb/ are populated by
    // scripts/pull-assets.sh and served directly by Next.js.
    return {
      fallback: [
        {
          source: '/wasm/:path*',
          destination: `${CDN_ORIGIN}/:path*`,
        },
        {
          source: '/kb/:path*',
          destination: `${CDN_ORIGIN}/:path*`,
        },
      ],
    };
  },
  async headers() {
    return [
      {
        source: '/wasm/:path(.+\\.wasm)',
        headers: [
          { key: 'Content-Type', value: 'application/wasm' },
          { key: 'Cache-Control', value: 'public, max-age=604800, immutable' },
        ],
      },
      {
        source: '/wasm/:path(.+\\.js)',
        headers: [
          { key: 'Content-Type', value: 'application/javascript' },
          { key: 'Cache-Control', value: 'public, max-age=604800, immutable' },
        ],
      },
      {
        source: '/kb/:path*',
        headers: [
          { key: 'Cache-Control', value: 'public, max-age=86400, immutable' },
        ],
      },
    ];
  },
};

export default withMDX(config);
