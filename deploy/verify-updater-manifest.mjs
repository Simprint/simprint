import fs from 'node:fs';

const EXPECTED_ASSET_TOKEN = {
  embedBootstrapper: 'embedBootstrapper',
  'fixed-runtime': 'fixed-runtime',
};

const REQUIRED_PLATFORMS = ['windows-x86_64', 'windows-aarch64', 'windows-i686'];
const [manifestPath, mode, releaseMetadataPath] = process.argv.slice(2);
const expectedAssetToken = EXPECTED_ASSET_TOKEN[mode];

if (!manifestPath || !expectedAssetToken || !releaseMetadataPath) {
  throw new Error(
    'Usage: node deploy/verify-updater-manifest.mjs <manifest-path> <embedBootstrapper|fixed-runtime> <release-metadata-path>'
  );
}

function readJson(path) {
  return JSON.parse(fs.readFileSync(path, 'utf8').replace(/^\uFEFF/, ''));
}

const manifest = readJson(manifestPath);
const release = readJson(releaseMetadataPath);
const platforms = manifest.platforms ?? {};
const assets = release.assets ?? [];

function resolveAsset(url) {
  const directMatch = assets.find(
    (asset) => asset.url === url || asset.browser_download_url === url
  );
  if (directMatch) {
    return directMatch;
  }

  const assetId = /\/releases\/assets\/(\d+)$/.exec(new URL(url).pathname)?.[1];
  return assetId ? assets.find((asset) => String(asset.id) === assetId) : undefined;
}

function verifyEntry(platform, entry) {
  if (!entry.signature?.trim()) {
    throw new Error(`${manifestPath} platform "${platform}" is missing its signature`);
  }

  const asset = entry.url ? resolveAsset(entry.url) : undefined;
  if (!asset) {
    throw new Error(`${manifestPath} platform "${platform}" does not reference a release asset`);
  }
  if (!asset.name.includes(expectedAssetToken)) {
    throw new Error(
      `${manifestPath} platform "${platform}" points to another mode's asset: "${asset.name}"`
    );
  }
}

for (const platform of REQUIRED_PLATFORMS) {
  const entry = platforms[platform];

  if (!entry) {
    throw new Error(`${manifestPath} is missing required updater platform "${platform}"`);
  }
  verifyEntry(platform, entry);
}

for (const [platform, entry] of Object.entries(platforms)) {
  if (platform.startsWith('windows-') && !REQUIRED_PLATFORMS.includes(platform)) {
    verifyEntry(platform, entry);
  }
}

console.log(
  `[verify-updater-manifest] ${manifestPath} contains ${mode} updates for x64, ARM64 and x86`
);
