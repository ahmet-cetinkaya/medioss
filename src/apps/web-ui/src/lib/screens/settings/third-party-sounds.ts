/** Attribution rows for the bundled sounds, derived from the one sound manifest. */

import manifest from "$repo/assets/sounds/soundscape-builtin/manifest.json";

type ManifestEntry = { category_name: string; name: string; source: string };

export type SoundAttribution = {
  title: string;
  /** Link to the upstream file, or null when the source is not `repo@ref path`. */
  sourceUrl: string | null;
};

export type SoundGroup = { category: string; sounds: SoundAttribution[] };

// "owner/repo@ref path/in/repo (processing notes)"
const UPSTREAM_SOURCE = /^([\w.-]+\/[\w.-]+)@(\S+) (\S+)/;

function sourceUrl(source: string): string | null {
  const match = UPSTREAM_SOURCE.exec(source);
  if (!match) return null;
  const [, repository, ref, path] = match;
  return `https://github.com/${repository}/blob/${ref}/${path}`;
}

export function groupSoundAttributions(entries: ManifestEntry[]): SoundGroup[] {
  const groups = new Map<string, SoundAttribution[]>();
  for (const entry of entries) {
    const sounds = groups.get(entry.category_name) ?? [];
    sounds.push({
      title: entry.name.replaceAll("-", " "),
      sourceUrl: sourceUrl(entry.source),
    });
    groups.set(entry.category_name, sounds);
  }
  return [...groups].map(([category, sounds]) => ({ category, sounds }));
}

export const soundGroups = groupSoundAttributions(manifest);
