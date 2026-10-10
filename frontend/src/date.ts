export function dateInput(value: string | Date) {
  const parts = new Intl.DateTimeFormat('sv-SE', { timeZone: 'Asia/Tokyo', year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', hourCycle: 'h23' }).formatToParts(new Date(value));
  const field = (name: string) => parts.find(p => p.type === name)?.value;
  return `${field('year')}-${field('month')}-${field('day')}T${field('hour')}:${field('minute')}`;
}
export function toApiDate(value: string) { return new Date(value + ':00+09:00').toISOString(); }
export function displayDate(value: string) { return new Intl.DateTimeFormat('en-GB', { timeZone: 'Asia/Tokyo', month: 'long', day: 'numeric', weekday: 'short', hour: '2-digit', minute: '2-digit', hourCycle: 'h23' }).format(new Date(value)); }
export function status(start: string, end: string) { const now = Date.now(); return new Date(end).getTime() <= now ? 'Completed' : new Date(start).getTime() <= now ? 'In progress' : new Date(start).getTime() <= now + 86400000 ? 'Starting soon' : 'Booked'; }
