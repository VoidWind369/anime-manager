import { addMessages, init, locale } from 'svelte-i18n';
import zhCN from './zh-CN.json';
import zhTW from './zh-TW.json';
import en from './en.json';
import ja from './ja.json';
import fr from './fr.json';
import ru from './ru.json';
import es from './es.json';
import ar from './ar.json';
import pt from './pt.json';

addMessages('zh-CN', zhCN);
addMessages('zh-TW', zhTW);
addMessages('en', en);
addMessages('ja', ja);
addMessages('fr', fr);
addMessages('ru', ru);
addMessages('es', es);
addMessages('ar', ar);
addMessages('pt', pt);

function getInitialLocale(): string {
  if (typeof window === 'undefined') return 'zh-CN';
  const saved = localStorage.getItem('app-language');
  if (saved) return saved;
  return 'zh-CN';
}

init({
  fallbackLocale: 'zh-CN',
  initialLocale: getInitialLocale(),
});

export function setLanguage(lang: string) {
  locale.set(lang);
  localStorage.setItem('app-language', lang);
}

export function getLanguage(): string {
  const current = getInitialLocale();
  return current;
}
