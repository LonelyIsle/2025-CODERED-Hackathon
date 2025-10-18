/** @type {import('next').NextConfig} */
const nextConfig = {
  // Generate static HTML in ./out for nginx
  output: 'export',

  // Required for static export when using <Image> etc.
  images: { unoptimized: true },

  // Optional but helps with simple static hosting
  trailingSlash: true,
};

export default nextConfig;