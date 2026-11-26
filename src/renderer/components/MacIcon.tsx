import React, { useState, useEffect } from 'react';
import type { DockAppItem } from '../../shared/constants';
import photosIcon from '../assets/photos-icon.jpg';
import finderIcon from '../assets/icons/finder.png';
import mailIcon from '../assets/icons/mail.png';
import musicIcon from '../assets/icons/music.png';
import settingsIcon from '../assets/icons/settings.png';

import { useClockAngles, useTimeStore } from '../store/timeStore';
import { useCachedImage } from '../hooks/useCachedImage';

// Glass artwork for the system apps. These are small 192px PNGs, so they stay light in memory.
const ICON_ART: Record<string, string | undefined> = {
  finder: finderIcon,
  mail: mailIcon,
  music: musicIcon,
  settings: settingsIcon,
};

interface MacIconProps {
  app: DockAppItem;
  size?: number;
  live?: boolean;
}

export const MacIcon: React.FC<MacIconProps> = ({ app, size = 48, live = true }) => {
  const isClock = app.iconType === 'clock';
  const isCalendar = app.iconType === 'calendar';

  const clockAngles = useClockAngles();
  const minuteKey = useTimeStore((s) => (isCalendar && live ? s.minuteKey : ''));

  const dateInfo = React.useMemo(() => {
    if (!isCalendar) return { month: 'JAN', day: '1', weekday: 'MON' };
    const now = new Date();
    const monthNames = ['JAN', 'FEB', 'MAR', 'APR', 'MAY', 'JUN', 'JUL', 'AUG', 'SEP', 'OCT', 'NOV', 'DEC'];
    const weekNames = ['SUN', 'MON', 'TUE', 'WED', 'THU', 'FRI', 'SAT'];
    return {
      month: monthNames[now.getMonth()],
      day: String(now.getDate()),
      weekday: weekNames[now.getDay()],
    };
  }, [isCalendar, minuteKey]);

  const [dynamicIcon, setDynamicIcon] = useState<string | null>(app.icon || null);

  useEffect(() => {
    let isMounted = true;
    if (app.iconType === 'custom' && app.path && !app.mono && !/^https?:/i.test(app.path) && window.electronAPI?.getAppIcon) {
      window.electronAPI
        .getAppIcon(app.path)
        .then((icon) => {
          if (isMounted && icon) {
            setDynamicIcon(icon);
          } else if (isMounted && app.icon) {
            setDynamicIcon(app.icon);
          }
        })
        .catch(() => {
          if (isMounted && app.icon) {
            setDynamicIcon(app.icon);
          }
        });
      return () => {
        isMounted = false;
      };
    } else if (app.icon) {
      setDynamicIcon(app.icon);
    }
  }, [app.icon, app.path, app.iconType]);

  const cachedIcon = useCachedImage(dynamicIcon || app.icon);

  const cornerRadius = Math.round(size * 0.22);

  const containerStyle: React.CSSProperties = {
    width: `${size}px`,
    height: `${size}px`,
    borderRadius: `${cornerRadius}px`,
    position: 'relative',
    overflow: 'hidden',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'center',
    userSelect: 'none',
    boxShadow: '0 4px 12px rgba(0, 0, 0, 0.35), inset 0 0.5px 0.5px rgba(255, 255, 255, 0.35)',
    flexShrink: 0,
  };

  if (app.iconType === 'calendar' && app.mono) {
    return (
      <div
        style={{
          ...containerStyle,
          background: 'linear-gradient(150deg, #3b3b40 0%, #151517 100%)',
          border: '0.5px solid rgba(255, 255, 255, 0.18)',
          flexDirection: 'column',
          gap: `${Math.round(size * 0.02)}px`,
        }}
      >
        <div
          style={{
            color: 'rgba(255, 255, 255, 0.62)',
            fontSize: `${Math.max(7, Math.round(size * 0.17))}px`,
            fontWeight: 700,
            letterSpacing: '0.4px',
            lineHeight: 1,
          }}
        >
          {dateInfo.weekday.charAt(0) + dateInfo.weekday.slice(1).toLowerCase()}
        </div>
        <div style={{ color: '#ffffff', fontSize: `${Math.round(size * 0.46)}px`, fontWeight: 600, lineHeight: 1 }}>
          {dateInfo.day}
        </div>
      </div>
    );
  }

  if (app.iconType === 'calendar') {
    return (
      <div style={{ ...containerStyle, background: '#ffffff', flexDirection: 'column' }}>
        <div
          style={{
            width: '100%',
            height: '28%',
            background: 'linear-gradient(180deg, #ff453a 0%, #d70015 100%)',
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: '#ffffff',
            fontSize: `${Math.max(7, Math.round(size * 0.16))}px`,
            fontWeight: 700,
            letterSpacing: '0.5px',
          }}
        >
          {dateInfo.month}
        </div>
        <div
          style={{
            flex: 1,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: '#1c1c1e',
            fontSize: `${Math.round(size * 0.44)}px`,
            fontWeight: 300,
            lineHeight: 1,
          }}
        >
          {dateInfo.day}
        </div>
      </div>
    );
  }

  if (app.iconType === 'clock') {
    const secDeg = clockAngles.second;
    const minDeg = clockAngles.minute;
    const hourDeg = clockAngles.hour;

    return (
      <div
        style={{
          ...containerStyle,
          background: 'linear-gradient(135deg, #2c2c2e 0%, #1c1c1e 100%)',
        }}
      >
        <svg width={size} height={size} viewBox="0 0 100 100">
          <circle cx="50" cy="50" r="46" fill="#1c1c1e" stroke="rgba(255,255,255,0.15)" strokeWidth="1.5" />
          {[0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330].map((deg) => (
            <line
              key={deg}
              x1="50"
              y1="10"
              x2="50"
              y2={deg % 90 === 0 ? '16' : '13'}
              stroke="rgba(255,255,255,0.6)"
              strokeWidth={deg % 90 === 0 ? '3' : '1.5'}
              transform={`rotate(${deg} 50 50)`}
            />
          ))}
          <line
            x1="50"
            y1="50"
            x2="50"
            y2="28"
            stroke="#ffffff"
            strokeWidth="4"
            strokeLinecap="round"
            transform={`rotate(${hourDeg} 50 50)`}
          />
          <line
            x1="50"
            y1="50"
            x2="50"
            y2="18"
            stroke="#ffffff"
            strokeWidth="2.5"
            strokeLinecap="round"
            transform={`rotate(${minDeg} 50 50)`}
          />
          <line
            x1="50"
            y1="56"
            x2="50"
            y2="15"
            stroke="#ff453a"
            strokeWidth="1.5"
            strokeLinecap="round"
            transform={`rotate(${secDeg} 50 50)`}
          />
          <circle cx="50" cy="50" r="3.5" fill="#ff453a" />
        </svg>
      </div>
    );
  }

  if (app.iconType === 'safari') {
    return (
      <div
        style={{
          ...containerStyle,
          background: 'linear-gradient(135deg, #ffffff 0%, #e0e0e0 100%)',
        }}
      >
        <svg width={size * 0.88} height={size * 0.88} viewBox="0 0 100 100">
          <defs>
            <linearGradient id="safariGrad" x1="0%" y1="0%" x2="100%" y2="100%">
              <stop offset="0%" stopColor="#29b6f6" />
              <stop offset="100%" stopColor="#0277bd" />
            </linearGradient>
          </defs>
          <circle cx="50" cy="50" r="44" fill="url(#safariGrad)" />
          {[0, 30, 60, 90, 120, 150, 180, 210, 240, 270, 300, 330].map((d) => (
            <line
              key={d}
              x1="50"
              y1="12"
              x2="50"
              y2={d % 90 === 0 ? '18' : '15'}
              stroke="rgba(255,255,255,0.7)"
              strokeWidth={d % 90 === 0 ? '2.5' : '1.5'}
              transform={`rotate(${d} 50 50)`}
            />
          ))}
          <polygon points="50,14 42,50 50,46" fill="#ff3b30" />
          <polygon points="50,14 58,50 50,46" fill="#d70015" />
          <polygon points="50,86 42,50 50,54" fill="#ffffff" />
          <polygon points="50,86 58,50 50,54" fill="#e0e0e0" />
          <circle cx="50" cy="50" r="4" fill="#ffffff" />
        </svg>
      </div>
    );
  }

  if (app.iconType === 'terminal') {
    return (
      <div
        style={{
          ...containerStyle,
          background: 'linear-gradient(180deg, #3a3a3c 0%, #1c1c1e 100%)',
          border: '0.5px solid rgba(255, 255, 255, 0.2)',
        }}
      >
        <svg width={size * 0.72} height={size * 0.72} viewBox="0 0 100 100">
          <rect x="6" y="10" width="88" height="80" rx="12" fill="#000000" />
          <path
            d="M22 32 L40 48 L22 64"
            stroke="#30d158"
            strokeWidth="7"
            strokeLinecap="round"
            strokeLinejoin="round"
            fill="none"
          />
          <line
            x1="48"
            y1="64"
            x2="74"
            y2="64"
            stroke="#ffffff"
            strokeWidth="7"
            strokeLinecap="round"
          />
        </svg>
      </div>
    );
  }

  const artwork = ICON_ART[app.iconType];
  if (artwork) {
    return (
      <div style={{ ...containerStyle, background: 'transparent', boxShadow: 'none', overflow: 'visible' }}>
        <img
          src={artwork}
          alt={app.name}
          width={size}
          height={size}
          style={{ width: `${size}px`, height: `${size}px`, display: 'block' }}
          draggable={false}
        />
      </div>
    );
  }

  if (app.iconType === 'photos') {
    return (
      <div
        style={{
          ...containerStyle,
          background: 'transparent',
          boxShadow: 'none',
        }}
      >
        <img
          src={photosIcon}
          alt="Photos"
          style={{
            width: `${size}px`,
            height: `${size}px`,
            borderRadius: `${cornerRadius}px`,
            objectFit: 'cover',
            filter: 'drop-shadow(0 4px 12px rgba(0, 0, 0, 0.35))',
          }}
          draggable={false}
        />
      </div>
    );
  }

  if (app.iconType === 'app-store') {
    return (
      <div
        style={{
          ...containerStyle,
          background: 'linear-gradient(135deg, #0a84ff 0%, #0051ba 100%)',
        }}
      >
        <svg width={size * 0.65} height={size * 0.65} viewBox="0 0 100 100">
          <line x1="50" y1="16" x2="22" y2="76" stroke="#ffffff" strokeWidth="10" strokeLinecap="round" />
          <line x1="50" y1="16" x2="78" y2="76" stroke="#ffffff" strokeWidth="10" strokeLinecap="round" />
          <line x1="26" y1="56" x2="74" y2="56" stroke="#ffffff" strokeWidth="10" strokeLinecap="round" />
        </svg>
      </div>
    );
  }

  const iconToRender = cachedIcon;

  if (iconToRender) {
    if (app.fill) {
      return (
        <div
          style={{
            ...containerStyle,
            background: 'linear-gradient(150deg, #3b3b40 0%, #151517 100%)',
            border: '0.5px solid rgba(255, 255, 255, 0.2)',
          }}
        >
          <img
            src={iconToRender}
            alt={app.name}
            style={{
              width: '100%',
              height: '100%',
              objectFit: 'cover',
              filter: app.mono ? 'grayscale(1)' : 'none',
            }}
            draggable={false}
          />
        </div>
      );
    }
    return (
      <div
        style={{
          ...containerStyle,
          background: app.mono
            ? 'linear-gradient(150deg, #3b3b40 0%, #151517 100%)'
            : 'linear-gradient(135deg, rgba(255, 255, 255, 0.14) 0%, rgba(255, 255, 255, 0.05) 100%)',
          backdropFilter: 'blur(20px)',
          WebkitBackdropFilter: 'blur(20px)',
          border: '0.5px solid rgba(255, 255, 255, 0.2)',
        }}
      >
        <img
          src={iconToRender}
          alt={app.name}
          style={{
            width: `${Math.round(size * (app.mono ? 0.62 : 0.74))}px`,
            height: `${Math.round(size * (app.mono ? 0.62 : 0.74))}px`,
            objectFit: 'contain',
            filter: `${app.mono ? 'grayscale(1) ' : ''}drop-shadow(0 2px 5px rgba(0, 0, 0, 0.4))`,
          }}
          draggable={false}
        />
      </div>
    );
  }

  const initial = app.name ? app.name.charAt(0).toUpperCase() : 'A';
  const customColor = app.color || 'linear-gradient(135deg, #636366 0%, #3a3a3c 100%)';

  return (
    <div
      style={{
        ...containerStyle,
        background: customColor.includes('gradient')
          ? customColor
          : `linear-gradient(135deg, ${customColor} 0%, #1c1c1e 100%)`,
      }}
    >
      <span
        style={{
          color: '#ffffff',
          fontSize: `${Math.round(size * 0.45)}px`,
          fontWeight: 700,
          letterSpacing: '-0.5px',
        }}
      >
        {initial}
      </span>
    </div>
  );
};
export default MacIcon;
