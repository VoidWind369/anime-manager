import { readFile } from '@tauri-apps/plugin-fs';

/**
 * 卡片封面缩略图尺寸（2x 适配 retina 屏）。
 * 卡片实际宽度约 200px、比例 3:4，取 2 倍像素保证清晰。
 */
const THUMB_WIDTH = 400;
const THUMB_HEIGHT = 533;
const THUMB_QUALITY = 0.85;

async function bytesToDataUrl(uint8: Uint8Array): Promise<string> {
  let mimeType = 'image/jpeg';
  if (uint8.length >= 4) {
    const header = uint8.subarray(0, 4);
    if (header[0] === 0x89 && header[1] === 0x50 && header[2] === 0x4E && header[3] === 0x47) {
      mimeType = 'image/png';
    } else if (uint8.length >= 12 && header[0] === 0x52 && header[1] === 0x49 && header[2] === 0x46 && header[3] === 0x46) {
      const webpHeader = uint8.subarray(8, 12);
      if (webpHeader[0] === 0x57 && webpHeader[1] === 0x45 && webpHeader[2] === 0x42 && webpHeader[3] === 0x50) {
        mimeType = 'image/webp';
      }
    } else if (header[0] === 0x47 && header[1] === 0x49 && header[2] === 0x46 && header[3] === 0x38) {
      mimeType = 'image/gif';
    } else if (header[0] === 0xFF && header[1] === 0xD8) {
      mimeType = 'image/jpeg';
    }
  }

  let binary = '';
  const chunkSize = 8192;
  for (let i = 0; i < uint8.length; i += chunkSize) {
    const chunk = uint8.subarray(i, i + chunkSize);
    binary += String.fromCharCode.apply(null, chunk as unknown as number[]);
  }
  const base64 = btoa(binary);
  return `data:${mimeType};base64,${base64}`;
}

/**
 * 将原图 data URL 缩放为卡片封面缩略图，减小内存占用、加快渲染。
 */
function createThumbnail(origDataUrl: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const img = new Image();
    img.onload = () => {
      const canvas = document.createElement('canvas');
      canvas.width = THUMB_WIDTH;
      canvas.height = THUMB_HEIGHT;
      const ctx = canvas.getContext('2d');
      if (!ctx) {
        resolve(origDataUrl);
        return;
      }

      // 按 cover 模式裁剪，与 CSS object-fit: cover 行为一致
      const srcRatio = img.width / img.height;
      const dstRatio = THUMB_WIDTH / THUMB_HEIGHT;
      let sx = 0, sy = 0, sw = img.width, sh = img.height;
      if (srcRatio > dstRatio) {
        sw = img.height * dstRatio;
        sx = (img.width - sw) / 2;
      } else {
        sh = img.width / dstRatio;
        sy = (img.height - sh) / 2;
      }

      ctx.drawImage(img, sx, sy, sw, sh, 0, 0, THUMB_WIDTH, THUMB_HEIGHT);
      resolve(canvas.toDataURL('image/jpeg', THUMB_QUALITY));
    };
    img.onerror = () => reject(new Error('Failed to decode image'));
    img.src = origDataUrl;
  });
}

/**
 * 读取本地封面文件，生成卡片缩略图后返回。
 * 不做内存缓存——组件层 coverCache 已承担同会话内复用。
 */
export async function loadCoverImage(filePath: string): Promise<string | null> {
  if (!filePath) return null;

  try {
    const bytes = await readFile(filePath);
    const uint8 = new Uint8Array(bytes);
    const origDataUrl = await bytesToDataUrl(uint8);
    const thumb = await createThumbnail(origDataUrl);
    return thumb;
  } catch (e: any) {
    console.error(`Failed to load cover ${filePath}: ${e.message || e}`);
    return null;
  }
}
