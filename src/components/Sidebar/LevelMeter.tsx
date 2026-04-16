// Copyright (c) 2026 Randall Rosas (Slategray). All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// LEVEL METER
// UI component for displaying real-time signal levels.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

import { useEffect, useRef, useState } from 'react';
import './LevelMeter.css';

interface LevelMeterProps {
  db: number;
  label?: string;
}

export function LevelMeter({ db, label }: LevelMeterProps) {
  const peakRef = useRef(-100);
  // Initialize in a way that avoids impure calls during render
  const lastPeakTimeRef = useRef(0);
  const [, forceUpdate] = useState(0);
  const [displayPeak, setDisplayPeak] = useState(-100);

  // Initialize time only once
  useEffect(() => {
    lastPeakTimeRef.current = Date.now();
  }, []);

  // Track peak when db rises
  useEffect(() => {
    if (db > peakRef.current) {
      peakRef.current = db;
      lastPeakTimeRef.current = Date.now();
      setTimeout(() => {
        setDisplayPeak(db);
      }, 0);
    }
  }, [db]);

  // Decay loop - runs on an interval so peak falls even when db is steady
  useEffect(() => {
    const timer = setInterval(() => {
      if (Date.now() - lastPeakTimeRef.current > 1000) {
        const next = Math.max(-100, peakRef.current - 0.5);
        if (next !== peakRef.current) {
          peakRef.current = next;
          setDisplayPeak(next);
          forceUpdate((n) => n + 1);
        }
      }
    }, 100);
    return () => clearInterval(timer);
  }, []);

  // Normalize dB (-60 to 0) to percentage (0 to 100)
  const normalize = (val: number) => {
    return Math.max(0, Math.min(100, ((val + 60) / 60) * 100));
  };

  const percent = normalize(db);
  const peakPercent = normalize(displayPeak);

  return (
    <div className="level-meter-container">
      {label && <div className="meter-label">{label}</div>}
      <div className="meter-rail">
        <div
          className="meter-fill"
          style={{
            height: `${percent}%`,
            background: getMeterColor(db),
          }}
        />
        <div className="meter-peak" style={{ bottom: `${peakPercent}%` }} />
        <div className="meter-ticks">
          <span>0</span>
          <span>-6</span>
          <span>-12</span>
          <span>-24</span>
          <span>-48</span>
        </div>
      </div>
      <div className="db-value">{db > -60 ? `${db.toFixed(1)} dB` : '-inf'}</div>
    </div>
  );
}

function getMeterColor(db: number) {
  if (db > -3) return 'linear-gradient(to top, #4ade80, #facc15, #ef4444)';
  if (db > -12) return 'linear-gradient(to top, #4ade80, #facc15)';
  return '#4ade80';
}
