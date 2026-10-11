import { describe, it, expect } from 'vitest';
import { dateInput, toApiDate, status, displayDate, calendarTitle, calendarListDate } from './date';
describe('Japanese reservation times', () => {
  it('shows year, month, day, weekday and time in Japan across midnight', () => {
    expect(displayDate('2026-12-31T15:00:00Z')).toBe('2027/01/01 (Fri) 00:00');
    expect(calendarListDate({year:2027,month:0,day:1})).toBe('2027/01/01 (Fri)');
  });
  it('shows month headings and date ranges without including the exclusive end', () => {
    const start = new Date('2026-12-28T00:00:00+09:00');
    const end = new Date('2027-01-04T00:00:00+09:00');
    expect(calendarTitle(start,end,'timeGridWeek')).toBe('2026/12/28 (Mon) – 2027/01/03 (Sun)');
    expect(calendarTitle(start,end,'dayGridMonth')).toBe('2026/12');
    expect(calendarTitle(start,end,'timeGridDay')).toBe('2026/12/28 (Mon)');
  });
  it('converts UTC to a Japanese input including midnight', () => {
    expect(dateInput('2026-10-10T15:00:00Z')).toBe('2026-10-11T00:00');
    expect(toApiDate('2026-10-11T00:00')).toBe('2026-10-10T15:00:00.000Z');
  });
  it('accepts existing seconds and arbitrary offsets', () => {
    expect(dateInput('2026-10-10T10:30:45+02:00')).toBe('2026-10-10T17:30');
  });
  it('marks ended reservations without relying on calendar color', () => {
    expect(status('2000-01-01T00:00:00Z','2000-01-01T01:00:00Z')).toBe('Completed');
  });
});
