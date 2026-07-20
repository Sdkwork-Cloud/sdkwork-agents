export const getPresentationHtml = (mdRaw: string, theme: string) => {
  const slides = mdRaw.split(/^---\s*$/gm);
  const slidesHtml = slides.map(
    (slide) => `<section data-markdown><script type="text/template">\n${slide}\n</script></section>`
  ).join('\n');

  return `
<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/reveal.js/5.0.4/reset.min.css">
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/reveal.js/5.0.4/reveal.min.css">
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/reveal.js/5.0.4/theme/${theme}.min.css" id="theme">
  <!-- Highlight.js theme -->
  <link rel="stylesheet" href="https://cdnjs.cloudflare.com/ajax/libs/highlight.js/11.9.0/styles/monokai.min.css">
  <style>
    body, html { margin: 0; padding: 0; width: 100%; height: 100%; }
    .reveal { width: 100%; height: 100%; }
  </style>
</head>
<body>
  <div class="reveal">
    <div class="slides">
      ${slidesHtml}
    </div>
  </div>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/reveal.js/5.0.4/reveal.js"></script>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/reveal.js/5.0.4/plugin/markdown/markdown.js"></script>
  <script src="https://cdnjs.cloudflare.com/ajax/libs/reveal.js/5.0.4/plugin/highlight/highlight.js"></script>
  <script>
    Reveal.initialize({
      hash: window.location.protocol !== 'about:',
      plugins: [ RevealMarkdown, RevealHighlight ]
    });
  </script>
</body>
</html>
  `;
};
