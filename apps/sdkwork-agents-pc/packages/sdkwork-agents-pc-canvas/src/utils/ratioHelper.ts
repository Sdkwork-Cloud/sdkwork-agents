export function getNumericAspectRatio(ratioStr: string | undefined): number {
  if (!ratioStr) return 1.0;
  const parts = ratioStr.split(':');
  if (parts.length === 2) {
    const w = parseFloat(parts[0]);
    const h = parseFloat(parts[1]);
    if (w > 0 && h > 0) {
      return w / h; // width / height
    }
  }
  return 1.0;
}

export function getAdaptedHeight(nodeType: string, width: number, ratioStr: string | undefined): number {
  if (nodeType === 'image-gen' || nodeType === 'video-gen') {
    const numericRatio = getNumericAspectRatio(ratioStr);
    // Card height = (width / aspect_ratio) + header_height (37px)
    return Math.round(width / numericRatio) + 37;
  }
  return 250; // default/fallback for text, etc.
}
