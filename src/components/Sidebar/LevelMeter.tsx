import { useEffect, useRef } from 'react';
import './LevelMeter.css';

interface LevelMeterProps {
    db: number;
    label?: string;
}

export function LevelMeter({ db, label }: LevelMeterProps) {
    const peakRef = useRef(-100);
    const lastPeakTimeRef = useRef(Date.now());

    // Track peak with decay
    useEffect(() => {
        if (db > peakRef.current) {
            peakRef.current = db;
            lastPeakTimeRef.current = Date.now();
        } else if (Date.now() - lastPeakTimeRef.current > 1000) {
            // Slow decay after 1 second
            peakRef.current = Math.max(-100, peakRef.current - 0.5);
        }
    }, [db]);

    // Normalize dB (-60 to 0) to percentage (0 to 100)
    const normalize = (val: number) => {
        return Math.max(0, Math.min(100, ((val + 60) / 60) * 100));
    };

    const percent = normalize(db);
    const peakPercent = normalize(peakRef.current);

    return (
        <div className="level-meter-container">
            {label && <div className="meter-label">{label}</div>}
            <div className="meter-rail">
                <div
                    className="meter-fill"
                    style={{
                        height: `${percent}%`,
                        background: getMeterColor(db)
                    }}
                />
                <div
                    className="meter-peak"
                    style={{ bottom: `${peakPercent}%` }}
                />
                <div className="meter-ticks">
                    <span>0</span>
                    <span>-6</span>
                    <span>-12</span>
                    <span>-24</span>
                    <span>-48</span>
                </div>
            </div>
            <div className="db-value">{db > -60 ? `${db.toFixed(1)} dB` : "-inf"}</div>
        </div>
    );
}

function getMeterColor(db: number): string {
    if (db > -3) return 'linear-gradient(to top, #4ade80, #facc15, #ef4444)';
    if (db > -12) return 'linear-gradient(to top, #4ade80, #facc15)';
    return '#4ade80';
}
