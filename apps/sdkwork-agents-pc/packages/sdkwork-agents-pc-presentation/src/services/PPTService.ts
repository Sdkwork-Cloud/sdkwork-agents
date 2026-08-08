const defaultMarkdown = `## Title Slide

This is a Web PPT powered by Reveal.js

---

## Features

- Write Markdown
- Preview instantly
- Split screen mode

---

## Code Example

\`\`\`javascript
const greeting = "Hello PPT!";
console.log(greeting);
\`\`\`
`;

export class PPTService {
  /**
   * Get the default presentation markdown template.
   *
   * NOTE: the presentation module is a local DEMO surface. It renders a
   * hardcoded template in the browser and does not call any SDKWork API;
   * AI-generated slide decks are not part of the shipped product scope yet.
   */
  static async getDefaultMarkdown(): Promise<string> {
    await new Promise(resolve => setTimeout(resolve, 200));
    return defaultMarkdown;
  }

}
