<?xml version="1.0" encoding="UTF-8"?>
<xsl:stylesheet version="1.0" xmlns:xsl="http://www.w3.org/1999/XSL/Transform" xmlns:sitemap="http://www.sitemaps.org/schemas/sitemap/0.9">
  <xsl:output method="html" encoding="UTF-8"/>
  <xsl:template match="/">
    <html lang="en">
      <head>
        <meta charset="utf-8"/>
        <meta name="viewport" content="width=device-width, initial-scale=1"/>
        <title>Sitemap</title>
        <style>
          :root { color-scheme: light dark; font-family: system-ui, sans-serif; }
          body { max-width: 72rem; margin: 0 auto; padding: clamp(1.25rem, 4vw, 3rem); line-height: 1.5; }
          h1 { margin-bottom: .25rem; font-size: clamp(2rem, 5vw, 3.5rem); }
          p { margin-top: 0; color: #707070; }
          table { width: 100%; border-collapse: collapse; background: Canvas; }
          th, td { padding: .8rem 1rem; border-bottom: 1px solid color-mix(in srgb, CanvasText 18%, transparent); text-align: left; }
          th { font-size: .8rem; text-transform: uppercase; letter-spacing: .06em; }
          a { color: LinkText; overflow-wrap: anywhere; }
          @media (max-width: 42rem) { th:last-child, td:last-child { display: none; } th, td { padding-inline: .5rem; } }
        </style>
      </head>
      <body>
        <h1>Sitemap</h1>
        <p><xsl:value-of select="count(sitemap:urlset/sitemap:url)"/> published URLs</p>
        <table>
          <thead><tr><th>URL</th><th>Last modified</th></tr></thead>
          <tbody>
            <xsl:for-each select="sitemap:urlset/sitemap:url">
              <tr>
                <td><a href="{sitemap:loc}"><xsl:value-of select="sitemap:loc"/></a></td>
                <td><xsl:value-of select="sitemap:lastmod"/></td>
              </tr>
            </xsl:for-each>
          </tbody>
        </table>
      </body>
    </html>
  </xsl:template>
</xsl:stylesheet>
