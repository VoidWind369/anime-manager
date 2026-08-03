let ctx = null;
let buttonId = 'fetch-cover';

export function onLoad(pluginContext) {
  ctx = pluginContext;
  ctx.log.info('封面抓取器插件已加载');

  ctx.ui.addToolbarButton(buttonId, {
    icon: `<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>`,
    title: '抓取封面',
    position: 'right',
    order: 10,
    onClick: handleFetchCover,
  });
}

export function onUnload() {
  if (ctx) {
    ctx.ui.removeToolbarButton(buttonId);
    ctx.log.info('封面抓取器插件已卸载');
  }
}

export function onAnimeSelect(anime) {
  if (!ctx) return;
  ctx.log.info(`选中动漫: ${anime.title}`);
}

async function handleFetchCover() {
  if (!ctx) return;

  const animeList = ctx.app.getAnimeList();
  const noCover = animeList.filter(a => !a.cover_image);

  if (noCover.length === 0) {
    ctx.ui.showToast({ message: '所有动漫都已有封面', type: 'success' });
    return;
  }

  const container = document.createElement('div');
  container.style.cssText = 'max-height:400px;overflow-y:auto;';

  const list = document.createElement('div');
  list.style.cssText = 'display:flex;flex-direction:column;gap:8px;';

  for (const anime of noCover) {
    const row = document.createElement('div');
    row.style.cssText = 'display:flex;align-items:center;gap:10px;padding:8px;background:var(--surface-dim);border-radius:8px;';

    const name = document.createElement('span');
    name.textContent = anime.title;
    name.style.cssText = 'flex:1;font-size:0.88rem;color:var(--text-primary);';

    const btn = document.createElement('button');
    btn.textContent = '选择封面';
    btn.style.cssText = 'padding:4px 12px;border-radius:6px;border:1px solid var(--accent-300);background:var(--accent-50);color:var(--accent-600);cursor:pointer;font-size:0.82rem;';
    btn.onclick = async () => {
      try {
        const { open } = await import('@tauri-apps/plugin-dialog');
        const file = await open({
          multiple: false,
          filters: [{ name: '图片', extensions: ['jpg', 'jpeg', 'png', 'webp'] }],
        });
        if (file) {
          await ctx.backend.invoke('set_anime_cover', { animeId: anime.id, coverPath: file });
          ctx.ui.showToast({ message: `已设置 ${anime.title} 的封面`, type: 'success' });
          btn.textContent = '已设置';
          btn.disabled = true;
          btn.style.opacity = '0.5';
        }
      } catch (e) {
        ctx.log.error(`设置封面失败: ${e}`);
        ctx.ui.showToast({ message: `设置失败: ${e}`, type: 'error' });
      }
    };

    row.appendChild(name);
    row.appendChild(btn);
    list.appendChild(row);
  }

  container.appendChild(list);

  await ctx.ui.showModal({
    title: `抓取封面 (${noCover.length} 部动漫缺少封面)`,
    content: container,
    buttons: [
      { label: '关闭', type: 'secondary', onClick: () => {} },
    ],
  });
}
