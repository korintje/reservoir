import { describe, it, expect } from 'vitest';
import { dateInput, toApiDate, status } from './date';
describe('Japanese reservation times', () => {
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
