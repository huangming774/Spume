import type { Glyph } from "../data";

export function GlyphText({ text, g, className }: { text: string; g: Glyph; className?: string }) {
  return (
    <text
      className={className}
      textRendering="geometricPrecision"
      fontSize={g.size}
      transform={g.matrix ? `matrix(${g.matrix.join(" ")} 0 0)` : undefined}
      x={g.x.join(" ")}
      y={g.y.join(" ")}
    >
      {text}
    </text>
  );
}
