import { CanvasNode, Connection, CanvasGroup } from '../types';

interface PDFExportOptions {
  nodes: CanvasNode[];
  groups: CanvasGroup[];
  connections: Connection[];
  containerElement: HTMLDivElement | null;
  zoom: number;
  pan: { x: number; y: number };
  showToast: (msg: string, type: 'success' | 'error' | 'info' | 'loading') => void;
}

export const exportCanvasToPDF = async ({
  nodes,
  groups,
  connections,
  containerElement,
  zoom,
  pan,
  showToast
}: PDFExportOptions) => {
  if (!containerElement) return;
  if (nodes.length === 0) {
    showToast('画布上没有可以导出的节点', 'error');
    return;
  }

  showToast('正在生成多页 PDF 流程文档...', 'loading');

  try {
    const [{ jsPDF }, { toPng }] = await Promise.all([
      import('jspdf'),
      import('html-to-image'),
    ]);
    // 1. Initialize jsPDF in A4 portrait format
    const pdf = new jsPDF({
      orientation: 'p',
      unit: 'mm',
      format: 'a4'
    });

    const pageWidth = pdf.internal.pageSize.getWidth(); // 210mm
    const pageHeight = pdf.internal.pageSize.getHeight(); // 297mm

    // ================= PAGE 1: SYSTEM OVERVIEW MAP =================
    // Fill background with a dark premium color to match the editor UI
    pdf.setFillColor(13, 13, 14);
    pdf.rect(0, 0, pageWidth, pageHeight, 'F');

    // Accent line at the top
    pdf.setFillColor(6, 182, 212); // Cyan theme accent
    pdf.rect(0, 0, pageWidth, 4, 'F');

    // Title and Header
    pdf.setTextColor(255, 255, 255);
    pdf.setFont('helvetica', 'bold');
    pdf.setFontSize(22);
    pdf.text('FLOW CANVAS DESIGN', 15, 25);
    
    pdf.setFont('helvetica', 'normal');
    pdf.setFontSize(10);
    pdf.setTextColor(150, 150, 150);
    pdf.text(`EXPORT DATE: ${new Date().toLocaleString()}`, 15, 32);
    pdf.text(`TOTAL NODES: ${nodes.length}  |  TOTAL CONNECTIONS: ${connections.length}`, 15, 37);

    // Canvas Section Title
    pdf.setFont('helvetica', 'bold');
    pdf.setFontSize(12);
    pdf.setTextColor(6, 182, 212);
    pdf.text('CANVAS ARCHITECTURE OVERVIEW', 15, 52);

    // Capture the entire canvas viewport using html-to-image
    const canvasImgData = await toPng(containerElement, {
      cacheBust: true,
      backgroundColor: '#0d0d0e',
      pixelRatio: 2,
      filter: (node: any) => {
        // Exclude UI controls, tools panels, help popups, and zoom buttons
        if (node.classList) {
          if (
            node.classList.contains('no-export') ||
            node.classList.contains('no-drag') ||
            node.tagName === 'BUTTON' ||
            node.classList.contains('zoom-controls') ||
            node.classList.contains('minimap') ||
            node.classList.contains('toolbar-container')
          ) {
            return false;
          }
        }
        return true;
      }
    });

    // Embed full flowchart map in a landscape box
    const mapWidth = 180;
    const mapHeight = 110;
    const mapX = 15;
    const mapY = 58;

    // Draw a subtle border around the overview map
    pdf.setDrawColor(255, 255, 255, 0.1);
    pdf.setLineWidth(0.5);
    pdf.rect(mapX - 0.5, mapY - 0.5, mapWidth + 1, mapHeight + 1);
    
    pdf.addImage(canvasImgData, 'PNG', mapX, mapY, mapWidth, mapHeight);

    // Dynamic Tree Summary / Index Box
    const indexBoxY = 180;
    pdf.setFillColor(22, 22, 24);
    pdf.rect(15, indexBoxY, 180, 95, 'F');
    pdf.setDrawColor(255, 255, 255, 0.05);
    pdf.rect(15, indexBoxY, 180, 95);

    pdf.setFont('helvetica', 'bold');
    pdf.setFontSize(11);
    pdf.setTextColor(255, 255, 255);
    pdf.text('DOCUMENTATION INDEX', 22, indexBoxY + 12);

    pdf.setFont('helvetica', 'normal');
    pdf.setFontSize(9);
    pdf.setTextColor(180, 180, 180);
    
    let infoOffset = indexBoxY + 22;
    nodes.slice(0, 7).forEach((node, idx) => {
      let typeLabel = '[Flow Card]';
      if (node.type === 'text') typeLabel = '[Text Draft]';
      else if (node.type === 'image-gen') typeLabel = '[AI Image] ';
      else if (node.type === 'video-gen') typeLabel = '[AI Video] ';
      else if (node.type === 'sticky') typeLabel = '[Sticky]   ';

      pdf.text(`${idx + 1}.  ${typeLabel}  -  ${node.title || 'Untitled Node'} (${node.status || 'idle'})`, 22, infoOffset);
      infoOffset += 9;
    });

    if (nodes.length > 7) {
      pdf.setTextColor(110, 110, 110);
      pdf.text(`... and ${nodes.length - 7} more nodes documented in detail on the following pages.`, 22, infoOffset);
    }

    // Footnotes
    pdf.setFontSize(8);
    pdf.setTextColor(110, 110, 110);
    pdf.text(`Page 1 of ${nodes.length + 1}`, 180, 285);


    // ================= SUBSEQUENT PAGES: CARD DETAIL SHEET =================
    for (let i = 0; i < nodes.length; i++) {
      const node = nodes[i];
      pdf.addPage();

      // Light, high-readability page background
      pdf.setFillColor(250, 250, 252);
      pdf.rect(0, 0, pageWidth, pageHeight, 'F');

      // Top Dark Accent Banner
      pdf.setFillColor(18, 18, 20);
      pdf.rect(0, 0, pageWidth, 16, 'F');

      // Header Text
      pdf.setFont('helvetica', 'bold');
      pdf.setFontSize(9);
      pdf.setTextColor(6, 182, 212); // Theme cyan
      pdf.text('FLOW ENGINE CANVAS', 15, 10);

      pdf.setFont('helvetica', 'normal');
      pdf.setFontSize(8);
      pdf.setTextColor(140, 140, 140);
      pdf.text('DETAILED FLOWCARD INVENTORY', 58, 10);
      pdf.text(`Page ${i + 2} of ${nodes.length + 1}`, 180, 10);

      // Section Separator Line
      pdf.setDrawColor(229, 231, 235);
      pdf.setLineWidth(0.3);
      pdf.line(15, 26, 195, 26);

      // Node Title Header
      pdf.setFont('helvetica', 'bold');
      pdf.setFontSize(16);
      pdf.setTextColor(18, 18, 20);
      pdf.text(node.title || `Node #${i + 1}`, 15, 36);

      // Node Type Styled Badge
      const badgeColors: Record<string, { bg: [number, number, number], text: [number, number, number], label: string }> = {
        'text': { bg: [236, 254, 255], text: [8, 145, 178], label: 'CREATIVE DRAFT / TEXT' },
        'image-gen': { bg: [240, 253, 250], text: [13, 148, 136], label: 'AI IMAGE GENERATION' },
        'video-gen': { bg: [245, 243, 255], text: [124, 58, 237], label: 'AI VIDEO RENDER' },
        'sticky': { bg: [254, 249, 195], text: [161, 98, 7], label: 'ANNOTATION / STICKY NOTE' }
      };

      const badge = badgeColors[node.type] || { bg: [244, 244, 245], text: [113, 113, 122], label: 'FLOW CARD' };
      pdf.setFillColor(badge.bg[0], badge.bg[1], badge.bg[2]);
      pdf.rect(15, 41, 60, 6, 'F');
      pdf.setFont('helvetica', 'bold');
      pdf.setFontSize(7);
      pdf.setTextColor(badge.text[0], badge.text[1], badge.text[2]);
      pdf.text(badge.label, 18, 45.2);

      // 2-Column Responsive Layout
      const col1X = 15;
      const col2X = 110;
      const colWidth = 85;

      // ---------------- COLUMN 1: SELECTABLE METADATA & PARAGRAPH CONTENT ----------------
      pdf.setFont('helvetica', 'bold');
      pdf.setFontSize(10);
      pdf.setTextColor(100, 100, 100);
      pdf.text('NODE METADATA PROPERTIES', col1X, 58);

      // Metadata Table background
      pdf.setFillColor(243, 244, 246);
      pdf.rect(col1X, 62, colWidth, 42, 'F');
      pdf.setDrawColor(229, 231, 235);
      pdf.rect(col1X, 62, colWidth, 42);

      pdf.setFont('helvetica', 'bold');
      pdf.setFontSize(8);
      pdf.setTextColor(55, 65, 81);
      
      pdf.text('Node ID:', col1X + 4, 69);
      pdf.text('Status:', col1X + 4, 76);
      pdf.text('Position:', col1X + 4, 83);
      pdf.text('Dimensions:', col1X + 4, 90);
      
      pdf.setFont('helvetica', 'normal');
      pdf.setTextColor(17, 24, 39);
      pdf.text(node.id, col1X + 22, 69);
      pdf.text(node.status?.toUpperCase() || 'IDLE', col1X + 22, 76);
      pdf.text(`X: ${node.x}px, Y: ${node.y}px`, col1X + 22, 83);
      pdf.text(`${node.width}x${node.height} px`, col1X + 22, 90);

      // Extra params depending on Node Type
      if (node.type === 'image-gen' || node.type === 'video-gen') {
        pdf.setFont('helvetica', 'bold');
        pdf.setTextColor(55, 65, 81);
        pdf.text('Model / Aspect:', col1X + 4, 97);
        pdf.setFont('helvetica', 'normal');
        pdf.setTextColor(17, 24, 39);
        pdf.text(`${node.model || 'Gemini 2.5 Flash'} (${node.ratio || '16:9'})`, col1X + 26, 97);
      } else if (node.type === 'sticky') {
        pdf.setFont('helvetica', 'bold');
        pdf.setTextColor(55, 65, 81);
        pdf.text('Sticky Color:', col1X + 4, 97);
        pdf.setFont('helvetica', 'normal');
        pdf.setTextColor(17, 24, 39);
        pdf.text(node.color || 'yellow', col1X + 26, 97);
      } else {
        pdf.setFont('helvetica', 'bold');
        pdf.setTextColor(55, 65, 81);
        pdf.text('Connections:', col1X + 4, 97);
        pdf.setFont('helvetica', 'normal');
        pdf.setTextColor(17, 24, 39);
        const count = connections.filter(c => c.fromNodeId === node.id || c.toNodeId === node.id).length;
        pdf.text(`${count} Active links`, col1X + 26, 97);
      }

      // Readable Vector text area
      pdf.setFont('helvetica', 'bold');
      pdf.setFontSize(10);
      pdf.setTextColor(100, 100, 100);
      pdf.text('CORE CONTENT / PROMPT TEXT', col1X, 114);

      // Content text container
      const contentBoxHeight = 160;
      pdf.setDrawColor(229, 231, 235);
      pdf.setFillColor(255, 255, 255);
      pdf.rect(col1X, 118, colWidth, contentBoxHeight, 'DF');

      // Resolve content text
      const fullText = node.type === 'text' 
        ? (node.content || '') 
        : (node.type === 'sticky' ? (node.content || '') : (node.prompt || ''));

      pdf.setFont('helvetica', 'normal');
      pdf.setFontSize(8.5);
      pdf.setTextColor(31, 41, 55);

      if (fullText && fullText.trim()) {
        const splitText = pdf.splitTextToSize(fullText, colWidth - 8);
        const maxLines = Math.floor((contentBoxHeight - 8) / 4.5);
        const linesToPrint = splitText.slice(0, maxLines);
        
        let textY = 124;
        linesToPrint.forEach((line: string) => {
          pdf.text(line, col1X + 4, textY);
          textY += 4.5;
        });

        if (splitText.length > maxLines) {
          pdf.setFont('helvetica', 'italic');
          pdf.setTextColor(120, 120, 120);
          pdf.text('[Content truncated... See original editor for complete text]', col1X + 4, textY + 2);
        }
      } else {
        pdf.setFont('helvetica', 'italic');
        pdf.setTextColor(156, 163, 175);
        pdf.text('No content or prompt provided for this node.', col1X + 4, 124);
      }

      // ---------------- COLUMN 2: CRISP CARD VISUAL SNAPSHOT ----------------
      pdf.setFont('helvetica', 'bold');
      pdf.setFontSize(10);
      pdf.setTextColor(100, 100, 100);
      pdf.text('FLOWCARD DESIGN CROP', col2X, 58);

      const cardElement = document.getElementById(node.id);
      if (cardElement) {
        try {
          // Render precise card with 3x scale for beautiful zoomed quality
          const nodeImgData = await toPng(cardElement, {
            pixelRatio: 3,
            backgroundColor: 'transparent',
            style: {
              boxShadow: 'none' // Remove browser-side visual artifacts for clean PDF
            }
          });

          // Embed card crop image
          const imgWidth = colWidth;
          const imgHeight = Math.min((node.height / node.width) * imgWidth, 180);
          const imgX = col2X;
          const imgY = 62 + (216 - imgHeight) / 2; // Center inside the frame

          pdf.addImage(nodeImgData, 'PNG', imgX, imgY, imgWidth, imgHeight);
        } catch (cardErr) {
          console.error(`Failed to crop node ${node.id} image`, cardErr);
          // Fallback box
          pdf.setFillColor(243, 244, 246);
          pdf.rect(col2X, 62, colWidth, 100, 'F');
          pdf.setDrawColor(229, 231, 235);
          pdf.rect(col2X, 62, colWidth, 100);
          pdf.setFont('helvetica', 'italic');
          pdf.setFontSize(8.5);
          pdf.setTextColor(156, 163, 175);
          pdf.text('[Visual snapshot unavailable]', col2X + 22, 112);
        }
      } else {
        // Fallback box
        pdf.setFillColor(243, 244, 246);
        pdf.rect(col2X, 62, colWidth, 100, 'F');
        pdf.setDrawColor(229, 231, 235);
        pdf.rect(col2X, 62, colWidth, 100);
        pdf.setFont('helvetica', 'italic');
        pdf.setFontSize(8.5);
        pdf.setTextColor(156, 163, 175);
        pdf.text('[Flowcard DOM element not found]', col2X + 18, 112);
      }

      // Outline surrounding visual card column
      pdf.setDrawColor(229, 231, 235);
      pdf.rect(col2X, 62, colWidth, 216);
    }

    // 3. Save the document and trigger download
    const dateStr = new Date().toISOString().slice(0, 10);
    pdf.save(`flow-canvas-documentation-${dateStr}.pdf`);
    showToast('PDF 流程文档成功导出！', 'success');
  } catch (err) {
    console.error('Failed to generate PDF document:', err);
    showToast('导出 PDF 失败，请重试', 'error');
  }
};
