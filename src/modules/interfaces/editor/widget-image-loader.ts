export class WidgetImageLoader {
  public async load(file: File): Promise<string> {
    if (
      file.size > 149_900 === true ||
      ['image/png', 'image/jpeg', 'image/webp'].includes(file.type) === false
    ) {
      throw new Error('Use a PNG, JPEG or WebP up to 150 KB');
    }

    return new Promise((resolve, reject) => {
      const reader = new FileReader();

      reader.onload = (): void => {
        if (typeof reader.result === 'string') {
          resolve(reader.result);
        } else {
          reject(new Error('Could not read image'));
        }
      };

      reader.onerror = (): void => {
        reject(new Error('Could not read image'));
      };

      reader.readAsDataURL(file);
    });
  }
}
