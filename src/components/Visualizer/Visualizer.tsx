// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

interface VisualizerProps {
  isRunning: boolean;
  spectrum: number[];
  tonality: number[];
}

function LeylineLogo() {
  return (
    <div className="logo-container">
      <svg
        viewBox="0 0 100 100"
        className="leyline-svg"
        role="img"
        aria-label="Logo"
      >
        <defs>
          <linearGradient id="line-grad" x1="0%" y1="0%" x2="100%" y2="0%">
            <stop offset="0%" stopColor="transparent" />
            <stop offset="50%" stopColor="var(--color-mauve)" />
            <stop offset="100%" stopColor="transparent" />
          </linearGradient>
        </defs>
        <path
          d="M50 20 L80 50 L50 80 L20 50 Z"
          fill="none"
          stroke="var(--color-gunmetal)"
          strokeWidth="1.5"
          className="filter-diamond"
        />
        <line x1="10" y1="45" x2="90" y2="45" className="leyline line-1" />
        <line x1="10" y1="50" x2="90" y2="50" className="leyline line-2" />
        <line x1="10" y1="55" x2="90" y2="55" className="leyline line-3" />
      </svg>
      <span className="logo-text">READY</span>
    </div>
  );
}

/**
 * # CSS-Driven Visualizer
 * We use CSS transitions on simple DOM elements for visualization instead of Canvas/WebGL.
 *
 * ## Why?
 * For a simple 12-band analyzer, DOM manipulation is performant enough and
 * much easier to style/responsively layout than a fixed-size canvas.
 */
export function Visualizer({ isRunning, spectrum, tonality }: VisualizerProps) {
  const bars = [...Array(12)].map((_, i) => {
    const binValue = spectrum[i] || 0;
    const binTonality = tonality[i] || 0;
    // binValue is already normalized 0.0 to 1.0 (-60dB to 0dB)
    const height = isRunning ? binValue * 100 : 0;

    return (
      <div
        key={i}
        className="wave-bar"
        style={{
          height: `${height}%`,
          transition: "height 0.04s cubic-bezier(0.17, 0.67, 0.83, 0.67)",
          // Opacity is now driven by harmonicity (tonality)
          // Tonal sounds (voice) glow more than noise
          opacity: isRunning ? 0.2 + binTonality * 0.8 : 0.2,
          boxShadow:
            isRunning && binTonality > 0.5
              ? "0 0 5px var(--color-mauve)"
              : "none",
          flex: 1,
        }}
      ></div>
    );
  });

  return (
    <div className="visualizer-container">
      {isRunning ? <div className="bars-wrapper">{bars}</div> : <LeylineLogo />}
    </div>
  );
}
