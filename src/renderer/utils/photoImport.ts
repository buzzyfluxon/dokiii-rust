const MAX_SIDE = 1600;

const blobToBase64 = (blob: Blob): Promise<string> =>
  new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => resolve(String(reader.result).split(',')[1] || '');
    reader.onerror = () => reject(reader.error);
    reader.readAsDataURL(blob);
  });

export const isImageFile = (file: File): boolean =>
  file.type.startsWith('image/') || /\.(png|jpe?g|webp|gif|bmp)$/i.test(file.name);

export const hasFiles = (e: { dataTransfer: DataTransfer | null }): boolean =>
  Boolean(e.dataTransfer && Array.from(e.dataTransfer.types).includes('Files'));

const downscale = async (file: File): Promise<{ blob: Blob; ext: string } | null> => {
  try {
    const bitmap = await createImageBitmap(file);
    const scale = Math.min(1, MAX_SIDE / Math.max(bitmap.width, bitmap.height));
    const canvas = document.createElement('canvas');
    canvas.width = Math.max(1, Math.round(bitmap.width * scale));
    canvas.height = Math.max(1, Math.round(bitmap.height * scale));
    const ctx = canvas.getContext('2d');
    if (!ctx) return null;
    ctx.drawImage(bitmap, 0, 0, canvas.width, canvas.height);
    bitmap.close?.();
    const blob = await new Promise<Blob | null>((resolve) => canvas.toBlob(resolve, 'image/webp', 0.9));
    return blob ? { blob, ext: 'webp' } : null;
  } catch (_) {
    return null;
  }
};

export const importImageFile = async (file: File): Promise<string | null> => {
  if (!isImageFile(file)) return null;
  try {
    let blob: Blob = file;
    let ext = (file.name.split('.').pop() || 'png').toLowerCase();
    if (file.type !== 'image/gif' && ext !== 'gif') {
      const small = await downscale(file);
      if (small) {
        blob = small.blob;
        ext = small.ext;
      }
    }
    const data = await blobToBase64(blob);
    return (await window.electronAPI?.savePhoto?.(data, ext)) || null;
  } catch (_) {
    return null;
  }
};

export const pickImageFromDialog = (): Promise<string | null> =>
  new Promise((resolve) => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = 'image/png,image/jpeg,image/webp,image/gif,image/bmp';
    input.style.display = 'none';
    const done = (value: string | null) => {
      input.remove();
      resolve(value);
    };
    input.onchange = async () => {
      const file = input.files?.[0];
      done(file ? await importImageFile(file) : null);
    };
    input.addEventListener('cancel', () => done(null));
    document.body.appendChild(input);
    input.click();
  });
