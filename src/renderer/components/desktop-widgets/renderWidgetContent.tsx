import React from 'react';
import { DesktopWidgetItem } from '../../../shared/constants';
import { WorldClockWidget } from './WorldClockWidget';
import { DateDayWidget } from './DateDayWidget';
import { SystemControlsWidget } from './SystemControlsWidget';
import { SystemMonitorWidget } from './SystemMonitorWidget';
import { BatteryWidget } from './BatteryWidget';
import { AnalogClockWidget } from './AnalogClockWidget';
import { CalendarMonthWidget } from './CalendarMonthWidget';
import { MoonPhaseWidget } from './MoonPhaseWidget';
import { CustomTextWidget } from './CustomTextWidget';
import { QuoteCardWidget } from './QuoteCardWidget';
import { CountdownWidget } from './CountdownWidget';
import { GoalsWidget } from './GoalsWidget';
import { CarCardWidget } from './CarCardWidget';
import { TravelCardWidget } from './TravelCardWidget';
import { BookCardWidget } from './BookCardWidget';
import { MovieCardWidget } from './MovieCardWidget';
import { WeatherWidget } from './WeatherWidget';
import { BigDateWidget } from './BigDateWidget';
import { DigitalClockWidget } from './DigitalClockWidget';
import { PhotoCardWidget } from './PhotoCardWidget';
import { MusicCardWidget } from './MusicCardWidget';
import { AppIconWidget } from './AppIconWidget';
import { QuickControlsWidget } from './QuickControlsWidget';

export const renderDesktopWidgetContent = (
  item: DesktopWidgetItem,
  dims: { width: number; height: number },
  onUpdateData: (patch: Record<string, any>) => void
) => {
  switch (item.type) {
    case 'weather':
      return <WeatherWidget width={dims.width} height={dims.height} city={item.customTitle} />;
    case 'big-date':
      return <BigDateWidget width={dims.width} height={dims.height} />;
    case 'digital-clock':
      return (
        <DigitalClockWidget
          width={dims.width}
          height={dims.height}
          label={item.customTitle}
          timezone={item.data?.timezone}
        />
      );
    case 'photo-card':
      return <PhotoCardWidget width={dims.width} height={dims.height} data={item.data} />;
    case 'music-card':
      return <MusicCardWidget width={dims.width} height={dims.height} data={item.data} />;
    case 'quick-controls':
      return <QuickControlsWidget width={dims.width} height={dims.height} />;
    case 'app-icon':
      return <AppIconWidget width={dims.width} height={dims.height} label={item.customTitle} data={item.data} />;
    case 'world-clock':
      return <WorldClockWidget size={item.size} customTitle={item.customTitle} city={item.city} />;
    case 'date-day':
      return <DateDayWidget size={item.size} customTitle={item.customTitle} />;
    case 'system-controls':
      return <SystemControlsWidget size={item.size} />;
    case 'system-monitor':
      return <SystemMonitorWidget size={item.size} />;
    case 'battery':
      return <BatteryWidget size={item.size} />;
    case 'analog-clock':
      return <AnalogClockWidget size={item.size} />;
    case 'calendar':
      return <CalendarMonthWidget size={item.size} />;
    case 'moon-phase':
      return <MoonPhaseWidget size={item.size} />;
    case 'custom-text':
      return (
        <CustomTextWidget
          size={item.size}
          data={item.data}
          onUpdateData={(patch) => onUpdateData(patch)}
        />
      );
    case 'quote-card':
      return (
        <QuoteCardWidget
          size={item.size}
          data={item.data}
          onUpdateData={(patch) => onUpdateData(patch)}
        />
      );
    case 'countdown':
      return (
        <CountdownWidget
          size={item.size}
          data={item.data}
          onUpdateData={(patch) => onUpdateData(patch)}
        />
      );
    case 'goals':
      return (
        <GoalsWidget
          size={item.size}
          data={item.data}
          onUpdateData={(patch) => onUpdateData(patch)}
        />
      );
    case 'car-card':
      return (
        <CarCardWidget
          size={item.size}
          data={item.data}
          onUpdateData={(patch) => onUpdateData(patch)}
        />
      );
    case 'travel-card':
      return (
        <TravelCardWidget
          size={item.size}
          data={item.data}
          onUpdateData={(patch) => onUpdateData(patch)}
        />
      );
    case 'book-card':
      return (
        <BookCardWidget
          size={item.size}
          data={item.data}
          onUpdateData={(patch) => onUpdateData(patch)}
        />
      );
    case 'movie-card':
      return (
        <MovieCardWidget
          size={item.size}
          data={item.data}
          onUpdateData={(patch) => onUpdateData(patch)}
        />
      );
    default:
      return null;
  }
};
