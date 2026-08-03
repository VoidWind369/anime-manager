let ctx = null;
const btnId = 'anime-stats-btn';

export function onLoad(pluginContext) {
  ctx = pluginContext;
  ctx.log.info('动漫统计插件已加载');

  ctx.ui.addToolbarButton(btnId, {
    icon: `<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21.21 15.89A10 10 0 1 1 8 2.83"/><path d="M22 12A10 10 0 0 0 12 2v10z"/></svg>`,
    title: '动漫统计',
    position: 'right',
    order: 5,
    onClick: showStats,
  });
}

export function onUnload() {
  if (ctx) {
    ctx.ui.removeToolbarButton(btnId);
    ctx.log.info('动漫统计插件已卸载');
  }
}

export function onSettingsChange(settings) {
  ctx.log.info('设置已更新: ' + JSON.stringify(settings));
}

function getSettings() {
  return ctx.storage.get('settings') ?? {};
}

function getAnimeList() {
  return ctx.app.getAnimeList() ?? [];
}

function formatSize(bytes) {
  if (bytes === 0) return '0 B';
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return (bytes / Math.pow(1024, i)).toFixed(1) + ' ' + units[i];
}

function makeStatCard(label, value, color) {
  return `
    <div style="
      flex:1;min-width:100px;padding:14px 16px;
      background:linear-gradient(135deg, ${color}18, ${color}08);
      border:1px solid ${color}30;border-radius:12px;text-align:center;
    ">
      <div style="font-size:1.6rem;font-weight:700;color:${color};margin-bottom:4px;">
        ${value}
      </div>
      <div style="font-size:0.78rem;color:var(--text-secondary);">${label}</div>
    </div>
  `;
}

function makeBar(label, value, max, color) {
  const pct = max > 0 ? (value / max) * 100 : 0;
  return `
    <div style="display:flex;align-items:center;gap:10px;margin-bottom:8px;">
      <span style="width:80px;font-size:0.82rem;color:var(--text-secondary);text-align:right;">${label}</span>
      <div style="flex:1;height:20px;background:var(--surface-dim);border-radius:10px;overflow:hidden;">
        <div style="height:100%;width:${pct}%;background:${color};border-radius:10px;transition:width 0.5s ease;"></div>
      </div>
      <span style="width:30px;font-size:0.82rem;font-weight:600;color:var(--text-primary);">${value}</span>
    </div>
  `;
}

async function showStats() {
  const list = getAnimeList();
  const settings = getSettings();
  const maxGroups = parseInt(settings.maxGroups) || 8;
  const showSeasonChart = settings.showSeasonChart !== false;
  const colorTheme = settings.colorTheme || 'multi';

  let totalEpisodes = 0;
  let watchedEpisodes = 0;
  const seasons = {};
  const subtitleGroups = {};

  for (const a of list) {
    totalEpisodes += a.total_episodes || 0;
    watchedEpisodes += a.watched_episodes || 0;

    const s = a.season || 1;
    seasons[s] = (seasons[s] || 0) + 1;

    if (a.subtitle_group) {
      subtitleGroups[a.subtitle_group] = (subtitleGroups[a.subtitle_group] || 0) + 1;
    }
  }

  const completionRate = totalEpisodes > 0
    ? Math.round((watchedEpisodes / totalEpisodes) * 100)
    : 0;

  const seasonEntries = Object.entries(seasons).sort((a, b) => Number(a[0]) - Number(b[0]));
  const groupEntries = Object.entries(subtitleGroups).sort((a, b) => b[1] - a[1]).slice(0, maxGroups);
  const maxGroup = groupEntries.length > 0 ? groupEntries[0][1] : 1;

  let seasonColors;
  if (colorTheme === 'mono') {
    seasonColors = Array(6).fill('var(--accent-500)');
  } else if (colorTheme === 'gradient') {
    seasonColors = ['#ff5595', '#ff7bab', '#ffa0c4', '#ffc2d9', '#ffe0eb', '#fff0f5'];
  } else {
    seasonColors = ['#ec4899', '#8b5cf6', '#3b82f6', '#10b981', '#f59e0b', '#ef4444'];
  }
  const groupColor = colorTheme === 'mono' ? 'var(--accent-500)' : '#8b5cf6';

  const container = document.createElement('div');
  container.innerHTML = `
    <div style="display:flex;flex-wrap:wrap;gap:12px;margin-bottom:20px;">
      ${makeStatCard('动漫总数', list.length, '#ec4899')}
      ${makeStatCard('总集数', totalEpisodes, '#8b5cf6')}
      ${makeStatCard('已看集数', watchedEpisodes, '#3b82f6')}
      ${makeStatCard('完成率', completionRate + '%', '#10b981')}
    </div>

    ${showSeasonChart && seasonEntries.length > 0 ? `
      <div style="margin-bottom:16px;">
        <div style="font-size:0.88rem;font-weight:600;color:var(--text-primary);margin-bottom:10px;">季分布</div>
        ${seasonEntries.map(([s, count], i) =>
          makeBar(`第 ${s} 季`, count, list.length, seasonColors[i % seasonColors.length])
        ).join('')}
      </div>
    ` : ''}

    ${groupEntries.length > 0 ? `
      <div>
        <div style="font-size:0.88rem;font-weight:600;color:var(--text-primary);margin-bottom:10px;">字幕组统计</div>
        ${groupEntries.map(([name, count]) =>
          makeBar(name.length > 6 ? name.slice(0, 6) + '…' : name, count, maxGroup, groupColor)
        ).join('')}
      </div>
    ` : ''}
  `;

  await ctx.ui.showModal({
    title: '动漫库统计',
    content: container,
    width: 520,
    buttons: [
      { label: '关闭', type: 'secondary', onClick: () => {} },
    ],
  });
}
