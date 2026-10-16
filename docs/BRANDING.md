# UltraMusic — Continuous sound

Nouvelle direction : un monogramme U composé de deux rubans dont l’espace
négatif forme une onde continue. Palette bleu-vert et pervenche.

Fichier maître : `assets/logo-master.png`, image raster avec canal alpha, générée avec
l’outil imagegen intégré. Il ne s’agit pas d’un SVG ni d’une source vectorielle.
Le logo est intégré au README, à sa bannière, au fichier exécutable et à la zone de notification Windows.

## Fichiers distribués

| Fichier | Utilisation |
| --- | --- |
| `assets/logo-master.png` | Source raster approuvée, transparence conservée |
| `assets/logo.png` | Export carré de 1280 px avec marge pour la documentation |
| `assets/ultramusic.ico` | Icône Windows, tailles de 16 à 256 px |
| `assets/banner.png` | Bannière du README, 1280 × 400 px |
| `assets/banner.svg` | Mise en page éditable de la bannière, avec logo raster incorporé |

Le signe est généré avec imagegen et n'est pas une source vectorielle. Les
exports conservent sa géométrie et sa palette. La bannière SVG combine texte et
formes vectoriels avec une image PNG ; elle ne vectorise pas le logo.

L'icône est incorporée à l'exécutable par `build.rs` via `winresource`, uniquement
pendant la compilation. Fenêtre et zone de notification réutilisent la même
ressource Windows partagée, sans lecture d'image ou traitement périodique.

## Prompt initial

```text
Use case: logo-brand.
Create one finished premium brand symbol for UltraMusic, a lightweight native Windows music player whose philosophy is "Keep the audio. Stop the unnecessary work."
Design concept: "continuous sound": a distinctive bold U monogram formed from two interlocking, smoothly flowing bands, subtly suggesting an audio wave through carefully shaped negative space. A compact memorable silhouette, deliberate asymmetry and excellent optical balance. Think refined independent software brand, precise Swiss design sensibility, not a generic music app stock icon.
Deliverable: ONLY the symbol, centered on a truly transparent square canvas, generous consistent 15% clear space. No lettering, no wordmark, no slogan, no labels, no presentation sheet.
Geometry: front-facing flat 2D vector-like artwork, smooth intentional curves, substantial weight, wide open negative space that remains legible at small sizes. Limit to two clean solid colors: deep saturated teal #087F8C and vivid periwinkle #6866E8. The form should still work as a single-color silhouette. Crisp edges.
Avoid headphones, play triangles, musical notes, equalizer bars, rounded-square app tile background, decorative rings, thin strokes, gradients, gloss, shadows, 3D, mockups, textures, watermarks, or any borrowed recognizable brand symbol.
Make the mark feel authored and elegant, with the visual flow of a continuous listening experience rather than a collection of music-related pictograms.
```

## Instruction de raffinement

```text
Refine this exact UltraMusic logo for a clean master asset. Preserve the U silhouette, the two flowing interlocking bands, positions, proportions, and curved negative-space wave. Do not redesign the mark.
Clean every contour: remove stray blue/colored pixels, flecks, rough edge artifacts, tiny protrusions, and edge halos, especially on the inside left edge of the violet upper-right stroke and above its top. Make all curves impeccably smooth, antialiased, deliberate.
Replace subtle gradient/noise shading with perfectly flat solid fills: teal #087F8C and periwinkle #6866E8. Keep all open spaces and surrounding background genuinely transparent, no solid matte, no checkerboard baked in.
Keep the same square composition and padding, no text, no shadows, no mockup, no background, no extra elements.
```
