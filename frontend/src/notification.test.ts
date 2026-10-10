import { describe, expect, it } from 'vitest';
import { slackWarning } from './notification';
import type { Reservation } from './types';
const now = Date.parse('2026-10-10T00:00:00Z');
const at = (hours: number) => new Date(now + hours * 3600000).toISOString();
const booking = (start: number, end: number) => ({ start: at(start), end: at(end) } as Reservation);
describe('Slack warning before saving', () => {
  it('warns for a delayed start within 24 hours', () => expect(slackWarning(booking(2,4),at(3),at(4),false,now)).toContain('delaying'));
  it('warns for an earlier end within 24 hours', () => expect(slackWarning(booking(24,26),at(24),at(25),false,now)).toContain('earlier'));
  it('warns for an earlier end while in progress', () => expect(slackWarning(booking(-1,2),at(-1),at(1),false,now)).toContain('in progress'));
  it('does not warn for extensions, notes, distant or completed bookings', () => {
    expect(slackWarning(booking(2,4),at(2),at(5),false,now)).toBeNull();
    expect(slackWarning(booking(2,4),at(2),at(4),false,now)).toBeNull();
    expect(slackWarning(booking(25,28),at(26),at(27),false,now)).toBeNull();
    expect(slackWarning(booking(-2,-1),at(-2),at(-1),true,now)).toBeNull();
  });
});
