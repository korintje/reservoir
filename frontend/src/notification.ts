import type { Reservation } from './types';

export function slackWarning(old: Reservation | null, start: string, end: string, deleted = false, now = Date.now()): string | null {
  if (!old || new Date(old.end).getTime() <= now) return null;
  const oldStart = new Date(old.start).getTime();
  const oldEnd = new Date(old.end).getTime();
  const reasons: string[] = [];
  if (oldStart > now && oldStart <= now + 86400000) {
    if (deleted) reasons.push('You are cancelling a booking that starts within 24 hours.');
    else {
      if (new Date(start).getTime() > oldStart) reasons.push('You are delaying the start of a booking that starts within 24 hours.');
      if (new Date(end).getTime() < oldEnd) reasons.push('You are moving the end of a booking that starts within 24 hours earlier.');
    }
  } else if (oldStart <= now && new Date(end).getTime() < oldEnd) {
    reasons.push('You are moving the end of a booking in progress earlier.');
  }
  return reasons.length ? reasons.join(' ') + ' Notify the resource’s Slack channel manually. This app does not send a Slack message.' : null;
}
