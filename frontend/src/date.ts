export function dateInput(value: string | Date) {
  const parts = new Intl.DateTimeFormat('sv-SE', { timeZone: 'Asia/Tokyo', year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', hourCycle: 'h23' }).formatToParts(new Date(value));
  const field = (name: string) => parts.find(p => p.type === name)?.value;
  return `${field('year')}-${field('month')}-${field('day')}T${field('hour')}:${field('minute')}`;
}
export function toApiDate(value: string) { return new Date(value + ':00+09:00').toISOString(); }
export function weekday(value: string | Date) { return new Intl.DateTimeFormat('en-US', { timeZone:'Asia/Tokyo', weekday:'short' }).format(new Date(value)); }
export function calendarDate(value: string | Date, includeYear = true) {
  const date = dateInput(value).slice(0,10).replaceAll('-','/');
  return `${includeYear ? date : date.slice(5)} (${weekday(value)})`;
}
export function displayDate(value: string) { return `${calendarDate(value)} ${dateInput(value).slice(11)}`; }
export function calendarTitle(start: Date, end: Date, view: string) {
  if (view === 'dayGridMonth') return dateInput(start).slice(0,7).replace('-','/');
  if (view === 'timeGridDay') return calendarDate(start);
  return `${calendarDate(start)} – ${calendarDate(new Date(end.getTime()-1))}`;
}
export function calendarListDate(date: { year:number; month:number; day:number }) {
  const value = new Date(Date.UTC(date.year,date.month,date.day)-9*3600000);
  return calendarDate(value);
}
export function status(start: string, end: string) { const now = Date.now(); return new Date(end).getTime() <= now ? 'Completed' : new Date(start).getTime() <= now ? 'In progress' : new Date(start).getTime() <= now + 86400000 ? 'Starting soon' : 'Booked'; }
