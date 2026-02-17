import { useEffect, useRef, useState } from 'react';
import './Oscilloscope.css';

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
    const lastUpdate = useRef<number>(Date.now());

    // 1. Animation Loop & Physics
    useEffect(() => {
        const frameInterval = 25; // ~40fps
        const timer = setInterval(() => {
            const now = Date.now();
            lastUpdate.current = now;

            // Target waveform selection
            // Backend now applies Hann windowing, so endpoints are naturally zero
            const target = isRunning ? waveform : Array(64).fill(0);

            for (let i = 0; i < 64; i++) {
                // Target waveform selection
                // Backend now applies Hann windowing, so endpoints are naturally zero
                const targetVal = target[i] || 0;

                // Remove smoothing for instant, raw response as requested
                internalWaveform.current[i] = targetVal;
            }

            setRenderTrigger(t => t + 1);
        }, frameInterval);

        return () => clearInterval(timer);
    }, [waveform, isRunning]);

    // 2. Canvas Drawing
    useEffect(() => {
        const canvas = canvasRef.current;
        if (!canvas) return;

        const ctx = canvas.getContext('2d');
        if (!ctx) return;

        const width = canvas.width;
        const height = canvas.height;

        ctx.clearRect(0, 0, width, height);

        // Draw very subtle zero line
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.02)';
        ctx.lineWidth = 1;
        ctx.beginPath();
        ctx.moveTo(0, height / 2);
        ctx.lineTo(width, height / 2);
        ctx.stroke();

        const points = internalWaveform.current;
        const sliceWidth = width / (points.length - 1);

        // --- Pass 1: Neural Glow (Single Continuous Path) ---
        ctx.beginPath();
        ctx.strokeStyle = 'rgba(255, 255, 255, 0.4)';
        ctx.lineWidth = 3;
        ctx.shadowBlur = 18;
        ctx.shadowColor = 'rgba(255, 255, 255, 0.6)';

        ctx.moveTo(0, height / 2);
        for (let i = 0; i < points.length - 1; i++) {
            const currentX = i * sliceWidth;
            const nextX = (i + 1) * sliceWidth;
            const currentY = (points[i] * (height / 1.7)) + (height / 2);
            const nextY = (points[i + 1] * (height / 1.7)) + (height / 2);
            const xc = (currentX + nextX) / 2;
            const yc = (currentY + nextY) / 2;
            ctx.quadraticCurveTo(currentX, currentY, xc, yc);
        }
        ctx.stroke();

        // --- Pass 2: Variable Weight Core (Segmented for Dynamic Thickness) ---
        ctx.shadowBlur = 0;
        ctx.lineCap = 'round';
        ctx.lineJoin = 'round';

        // We use quadratic interpolation for segment endpoints to match the glow path exactly
        for (let i = 0; i < points.length - 1; i++) {
            const currentX = i * sliceWidth;
            const nextX = (i + 1) * sliceWidth;
            const currentY = (points[i] * (height / 1.7)) + (height / 2);
            const nextY = (points[i + 1] * (height / 1.7)) + (height / 2);

            // Critical: Midpoint interpolation for smooth segmentation
            const xc = (currentX + nextX) / 2;
            const yc = (currentY + nextY) / 2;

            // Previous Midpoint (Start of current segment)
            let prevXc = 0;
            let prevYc = height / 2;
            if (i > 0) {
                prevXc = ((i - 1) * sliceWidth + currentX) / 2;
                prevYc = (points[i - 1] * (height / 1.7) + height / 2 + currentY) / 2;
            }

            // Variable weight based on current point amplitude
            const absY = Math.abs(points[i]);
            const dynamicWeight = 0.8 + (absY * 2.8);

            ctx.lineWidth = dynamicWeight;
            ctx.strokeStyle = '#ffffff';

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
