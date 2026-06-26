export type DefaultAvatarKind = 'agent' | 'direct' | 'group' | 'user';

const AVATAR_COLORS: Record<DefaultAvatarKind, { accent: string; background: string; foreground: string }> = {
  agent: {
    accent: '#8b5cf6',
    background: '#261f39',
    foreground: '#e9ddff',
  },
  direct: {
    accent: '#22c55e',
    background: '#153322',
    foreground: '#d9fbe7',
  },
  group: {
    accent: '#38bdf8',
    background: '#132f3f',
    foreground: '#d9f3ff',
  },
  user: {
    accent: '#f59e0b',
    background: '#392714',
    foreground: '#fff0cf',
  },
};

function encodeSvg(svg: string): string {
  return `data:image/svg+xml;utf8,${encodeURIComponent(svg)}`;
}

function renderIcon(kind: DefaultAvatarKind, foreground: string, accent: string): string {
  if (kind === 'group') {
    return `
      <circle cx="40" cy="36" r="12" fill="${foreground}"/>
      <circle cx="25" cy="43" r="9" fill="${accent}"/>
      <circle cx="55" cy="43" r="9" fill="${accent}"/>
      <path d="M17 65c2.8-10.8 13.1-18 23-18s20.2 7.2 23 18" fill="none" stroke="${foreground}" stroke-width="7" stroke-linecap="round"/>
    `;
  }

  if (kind === 'agent') {
    return `
      <rect x="20" y="24" width="40" height="34" rx="12" fill="${foreground}"/>
      <circle cx="32" cy="40" r="4" fill="${accent}"/>
      <circle cx="48" cy="40" r="4" fill="${accent}"/>
      <path d="M33 50h14" stroke="${accent}" stroke-width="4" stroke-linecap="round"/>
      <path d="M40 18v-7" stroke="${foreground}" stroke-width="5" stroke-linecap="round"/>
      <circle cx="40" cy="9" r="4" fill="${accent}"/>
    `;
  }

  if (kind === 'direct') {
    return `
      <rect x="17" y="22" width="46" height="34" rx="14" fill="${foreground}"/>
      <path d="M30 56l-8 10v-13" fill="${foreground}"/>
      <path d="M29 38h22M29 48h14" stroke="${accent}" stroke-width="5" stroke-linecap="round"/>
    `;
  }

  return `
    <circle cx="40" cy="32" r="14" fill="${foreground}"/>
    <path d="M18 66c3.2-13 13-21 22-21s18.8 8 22 21" fill="${foreground}"/>
  `;
}

export function createDefaultAvatar(kind: DefaultAvatarKind): string {
  const colors = AVATAR_COLORS[kind];
  return encodeSvg(`
    <svg xmlns="http://www.w3.org/2000/svg" width="80" height="80" viewBox="0 0 80 80" role="img" aria-label="${kind} avatar">
      <rect width="80" height="80" rx="18" fill="${colors.background}"/>
      <circle cx="66" cy="14" r="8" fill="${colors.accent}" opacity="0.75"/>
      ${renderIcon(kind, colors.foreground, colors.accent)}
    </svg>
  `);
}

export function createDefaultMusicCover(): string {
  return encodeSvg(`
    <svg xmlns="http://www.w3.org/2000/svg" width="80" height="80" viewBox="0 0 80 80" role="img" aria-label="music cover">
      <rect width="80" height="80" rx="12" fill="#141414"/>
      <circle cx="40" cy="40" r="24" fill="#242424" stroke="#1db954" stroke-width="3"/>
      <circle cx="40" cy="40" r="7" fill="#1db954"/>
      <path d="M54 22v24" stroke="#1db954" stroke-width="4" stroke-linecap="round"/>
      <circle cx="54" cy="46" r="5" fill="#1db954"/>
    </svg>
  `);
}

export const DEFAULT_MUSIC_COVER_URL = createDefaultMusicCover();
