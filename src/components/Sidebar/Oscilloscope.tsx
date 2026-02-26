import { useEffect, useRef, useState } from "react";
import "./Oscilloscope.css";

interface OscilloscopeProps {
  waveform: number[];
  isRunning: boolean;
}

/**
 * # Oscilloscope Component
 * A high-performance, real-time waveform visualizer.
 * Refined with backend windowing support, "Liquid" smoothing, and neural glow.
 */
export function Oscilloscope({ waveform, isRunning }: OscilloscopeProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [renderTrigger, setRenderTrigger] = useState(0);

  // Persisted state for smoothing and decay
  const internalWaveform = useRef<number[]>(Array(64).fill(0));
  const waveformRef = useRef<number[]>(waveform);
  const isRunningRef = useRef<boolean>(isRunning);

  // Keep refs up-to-date without restarting the animation loop
  useEffect(() => { waveformRef.current = waveform; }, [waveform]);
  useEffect(() => { isRunningRef.current = isRunning; }, [isRunning]);

  // 1. Animation Loop & Physics - starts once, reads from refs
  useEffect(() => {
    const frameInterval = 25; // ~40fps
    const timer = setInterval(() => {
      // Target waveform selection
      // Backend now applies Hann windowing, so endpoints are naturally zero
      const target = isRunningRef.current ? waveformRef.current : Array(64).fill(0);

      for (let i = 0; i < 64; i++) {
        const targetVal = target[i] || 0;

        // Remove smoothing for instant, raw response as requested
        internalWaveform.current[i] = targetVal;
      }

      setRenderTrigger((t) => t + 1);
    }, frameInterval);

    return () => clearInterval(timer);
  }, []);

  // 2. Canvas Drawing
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // High DPI Scaling
    const dpr = window.devicePixelRatio || 1;
    const displayWidth = 300;
    const displayHeight = 130;

    if (canvas.width !== displayWidth * dpr) {
      canvas.width = displayWidth * dpr;
      canvas.height = displayHeight * dpr;
    }

    ctx.setTransform(1, 0, 0, 1, 0, 0); // Reset
    ctx.scale(dpr, dpr);
    ctx.clearRect(0, 0, displayWidth, displayHeight);

    // Draw very subtle zero line
    ctx.strokeStyle = "rgba(255, 255, 255, 0.02)";
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(0, displayHeight / 2);
    ctx.lineTo(displayWidth, displayHeight / 2);
    ctx.stroke();

    const points = internalWaveform.current;
    const sliceWidth = displayWidth / (points.length - 1);

    // --- Pass 1: Neural Glow (Single Continuous Path) ---
    ctx.beginPath();
    ctx.strokeStyle = "rgba(255, 255, 255, 0.2)";
    ctx.lineWidth = 3;
    ctx.shadowBlur = 10;
    ctx.shadowColor = "rgba(255, 255, 255, 0.3)";

    ctx.moveTo(0, displayHeight / 2);
    for (let i = 0; i < points.length - 1; i++) {
      const currentX = i * sliceWidth;
      const nextX = (i + 1) * sliceWidth;
      const currentY = points[i] * (displayHeight / 1.7) + displayHeight / 2;
      const nextY = points[i + 1] * (displayHeight / 1.7) + displayHeight / 2;
      const xc = (currentX + nextX) / 2;
      const yc = (currentY + nextY) / 2;
      ctx.quadraticCurveTo(currentX, currentY, xc, yc);
    }
    ctx.stroke();

    // --- Pass 2: Variable Weight Core (Segmented for Dynamic Thickness) ---
    ctx.shadowBlur = 0;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";

    // We use quadratic interpolation for segment endpoints to match the glow path exactly
    for (let i = 0; i < points.length - 1; i++) {
      const currentX = i * sliceWidth;
      const nextX = (i + 1) * sliceWidth;
      const currentY = points[i] * (displayHeight / 1.7) + displayHeight / 2;
      const nextY = points[i + 1] * (displayHeight / 1.7) + displayHeight / 2;

      // Critical: Midpoint interpolation for smooth segmentation
      const xc = (currentX + nextX) / 2;
      const yc = (currentY + nextY) / 2;

      // Previous Midpoint (Start of current segment)
      let prevXc = 0;
      let prevYc = displayHeight / 2;
      if (i > 0) {
        prevXc = ((i - 1) * sliceWidth + currentX) / 2;
        prevYc =
          (points[i - 1] * (displayHeight / 1.7) +
            displayHeight / 2 +
            currentY) /
          2;
      }

      // Variable weight based on current point amplitude
      const absY = Math.abs(points[i]);
      const dynamicWeight = 0.8 + absY * 2.8;

      ctx.lineWidth = dynamicWeight;
      ctx.strokeStyle = "#ffffff";

      ctx.beginPath();
      ctx.moveTo(prevXc, prevYc);
      ctx.quadraticCurveTo(currentX, currentY, xc, yc);
      ctx.stroke();
    }
  }, [renderTrigger]);

  return (
    <div className="oscilloscope-container animated">
      <canvas
        ref={canvasRef}
        width={300}
        height={130}
        className="oscope-canvas"
      />
    </div>
  );
}
