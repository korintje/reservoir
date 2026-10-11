import { useEffect, useRef, useState, type FormEvent } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useSearchParams } from 'react-router-dom';
import FullCalendar from '@fullcalendar/react';
import dayGridPlugin from '@fullcalendar/daygrid';
import timeGridPlugin from '@fullcalendar/timegrid';
import interactionPlugin from '@fullcalendar/interaction';
import listPlugin from '@fullcalendar/list';
import luxonPlugin from '@fullcalendar/luxon3';
import { Plus, ArrowUpRight, X, RefreshCw, Clock3, CalendarDays, List, ChevronLeft, ChevronRight } from 'lucide-react';
import { api } from './api';
import { slackWarning } from './notification';
import { dateInput, toApiDate, displayDate, status } from './date';
import type { Member, Resource, Reservation, Settings, Result } from './types';

type Editor = { reservation: Reservation | null; resource: number; member: number; start: string; end: string };
export default function ReservationPage() {
  const [params, setParams] = useSearchParams();
  const requestedResource = Number(params.get('resource') || params.get('machine') || 0);
  const requestedUser = Number(params.get('user') || 0);
  const resourceId = Number.isSafeInteger(requestedResource) && requestedResource > 0 ? requestedResource : 0;
  const userId = Number.isSafeInteger(requestedUser) && requestedUser > 0 ? requestedUser : 0;
  const [resourceSearch, setResourceSearch] = useState('');
  const [period, setPeriod] = useState({ from: '', until: '' });
  const [title, setTitle] = useState('');
  const [editor, setEditor] = useState<Editor | null>(null);
  const [notice, setNotice] = useState<Result | null>(null);
  const [view, setView] = useState(() => window.innerWidth < 700 ? 'listWeek' : 'timeGridWeek');
  const [calendarHeight, setCalendarHeight] = useState(() => Math.max(420, window.innerHeight - 145));
  useEffect(() => {
    const resize = () => setCalendarHeight(Math.max(420, window.innerHeight - 145));
    window.addEventListener('resize', resize);
    return () => window.removeEventListener('resize', resize);
  }, []);
  const calendar = useRef<FullCalendar>(null);
  const members = useQuery({ queryKey: ['users'], queryFn: () => api<Member[]>('/users') });
  const resources = useQuery({ queryKey: ['resources'], queryFn: () => api<Resource[]>('/resources') });
  const settings = useQuery({ queryKey: ['settings'], queryFn: () => api<Settings>('/settings') });
  useEffect(() => {
    if (!view.startsWith('timeGrid')) return;
    const frame = requestAnimationFrame(() => calendar.current?.getApi().scrollToTime('08:00:00'));
    return () => cancelAnimationFrame(frame);
  }, [calendarHeight, view, settings.data?.slot_minutes]);
  const reservations = useQuery({
    queryKey: ['reservations', period, resourceId],
    queryFn: () => api<Reservation[]>('/reservations?' + new URLSearchParams({
      ...period, ...(resourceId ? { resource_id: String(resourceId) } : {}),
    })), enabled: Boolean(period.from), refetchInterval: 60000,
  });
  const activeResources = resources.data?.filter(r => r.active) || [];
  const choose = (key: string, value: string) => { const next = new URLSearchParams(params); next.delete('machine'); value === '0' ? next.delete(key) : next.set(key, value); setParams(next); };
  const changeView = (value: string) => { setView(value); calendar.current?.getApi().changeView(value); };
  function create(start?: string, end?: string) {
    const slot = (settings.data?.slot_minutes || 30) * 60000;
    const rounded = new Date(Math.ceil(Date.now() / slot) * slot);
    setEditor({ reservation: null, resource: resourceId || activeResources[0]?.id || 0, member:members.data?.some(m => m.id === userId && m.active) ? userId : 0, start: dateInput(start || rounded), end: dateInput(end || new Date(rounded.getTime() + 3600000)) });
  }
  const loadError = members.error || resources.error || settings.error;
  return <>
    {notice && <div className={notice.notification_required ? 'notice warning' : 'notice success'} role="status"><div><strong>{notice.message}</strong>{notice.notification_required && <><p>{notice.notification_reason}。{settings.data?.notification_text}</p>{notice.slack_url && <a href={notice.slack_url} target="_blank" rel="noreferrer">Open Slack channel <ArrowUpRight size={15}/></a>}</>}</div><button className="icon-button" aria-label="Dismiss notification" onClick={() => setNotice(null)}><X size={18}/></button></div>}
    {loadError && <div className="notice error" role="alert">{loadError.message}<button onClick={() => { void members.refetch(); void resources.refetch(); void settings.refetch(); }}>Retry</button></div>}
    <div className="workspace">
      <aside className="resource-panel">
        <button className="primary new-booking-button" onClick={() => create()} disabled={!activeResources.length || !members.data?.some(m => m.active)}><Plus size={18}/>New booking</button>
        <div className="panel-label">Resources <span>{activeResources.length}</span></div>
        <select className="mobile-resource-select" aria-label="Resource filter" value={resourceId} onChange={e => choose('resource',e.target.value)}><option value={0}>All resources</option>{activeResources.map(r => <option key={r.id} value={r.id}>{r.resource_name}</option>)}</select>
        <div className="resource-directory"><input type="search" aria-label="Search resources" placeholder="Search resources…" value={resourceSearch} onChange={e => setResourceSearch(e.target.value)}/>
        <button className={'resource-button ' + (!resourceId ? 'selected' : '')} onClick={() => choose('resource', '0')}><span className="resource-dot all"/><span>All resources</span></button>
        <div className="resource-list" aria-label="Resources">{activeResources.filter(r => (r.resource_name+' '+r.custom_field).toLowerCase().includes(resourceSearch.toLowerCase())).map(r => { const i = activeResources.findIndex(item => item.id === r.id); return <button className={'resource-button ' + (resourceId === r.id ? 'selected' : '')} key={r.id} onClick={() => choose('resource', String(r.id))}><span className="resource-dot" style={{ background: colors[i % colors.length] }}/><span>{r.resource_name}<small>{r.custom_field || 'Lab resource'}</small></span>{resourceId === r.id && <span className="selection-mark"/>}</button>; })}</div></div>
        {!resources.isPending && !activeResources.length && <p className="muted">No resources yet. Add resources in Settings.</p>}
        <label className="filter-label">Default booking member<select value={userId} onChange={e => choose('user', e.target.value)}><option value={0}>Select a member</option>{members.data?.map(m => <option key={m.id} value={m.id} disabled={!m.active}>{m.user_name}{!m.active ? ' (inactive)' : ''}</option>)}</select></label>
        <div className="rules"><Clock3 size={18}/><strong>Booking rules</strong><p>Book up to {settings.data?.advance_days || 15} days ahead. No overlapping bookings for the same resource.</p><p>Completed bookings cannot be edited. Notify Slack for short-notice changes.</p></div>
        {settings.data?.contact_text && <p className="contact-text">{settings.data.contact_text}</p>}
      </aside>
      <section className="calendar-panel"><div className="calendar-toolbar"><div className="calendar-nav"><button className="icon-button" aria-label="Previous period" onClick={() => calendar.current?.getApi().prev()}><ChevronLeft size={20}/></button><button className="icon-button" aria-label="Next period" onClick={() => calendar.current?.getApi().next()}><ChevronRight size={20}/></button><button className="subtle" onClick={() => calendar.current?.getApi().today()}>Today</button><h2>{title}</h2></div><div className="view-switch"><button aria-label="Refresh" className="icon-button" onClick={() => void reservations.refetch()}><RefreshCw size={17} className={reservations.isFetching ? 'spinning' : ''}/></button>{[['timeGridDay','Day'],['timeGridWeek','Week'],['dayGridMonth','Month'],['listWeek','List']].map(([value, label]) => <button key={value} className={view === value ? 'active' : ''} onClick={() => changeView(value)}>{value === 'listWeek' ? <List size={15}/> : null}{label}</button>)}</div></div>
        {reservations.error && <div className="notice error" role="alert">{reservations.error.message}<button onClick={() => void reservations.refetch()}>Retry</button></div>}
        {reservations.isFetching && <p className="loading-text" role="status">Loading bookings…</p>}
        <FullCalendar ref={calendar} plugins={[dayGridPlugin,timeGridPlugin,interactionPlugin,listPlugin,luxonPlugin]} locale="en-gb" timeZone="Asia/Tokyo" initialView={view} headerToolbar={false} height={view === 'listWeek' ? 'auto' : calendarHeight} firstDay={1} expandRows allDaySlot={false} nowIndicator eventMinHeight={68} editable={false} selectable={Boolean(activeResources.length && members.data?.some(m => m.active))} selectLongPressDelay={250} slotMinTime="00:00:00" slotMaxTime="24:00:00" scrollTime="08:00:00" slotDuration={settings.data?.slot_minutes === 60 ? '01:00:00' : '00:' + String(settings.data?.slot_minutes || 30).padStart(2,'0') + ':00'} slotLabelInterval="01:00:00" eventTimeFormat={{ hour:'2-digit',minute:'2-digit',hour12:false }} slotLabelFormat={{ hour:'2-digit',minute:'2-digit',hour12:false }}
          datesSet={info => { setTitle(info.view.title); setPeriod({ from:info.start.toISOString(), until:info.end.toISOString() }); }}
          select={info => { create(info.startStr, info.endStr); calendar.current?.getApi().unselect(); }}
          events={(reservations.data || []).map(r => ({ id:String(r.id), title:r.user_name + (resourceId ? '' : ' · ' + r.resource_name), start:r.start, end:r.end, backgroundColor:colors[Math.max(0, activeResources.findIndex(s => s.id === r.resource_id)) % colors.length], borderColor:'transparent', extendedProps:{ state:status(r.start,r.end), description:r.description } }))}
          eventClick={info => { const r = reservations.data?.find(item => item.id === Number(info.event.id)); if (r) setEditor({ reservation:r,resource:r.resource_id,member:r.user_id,start:dateInput(r.start),end:dateInput(r.end) }); }}
          eventContent={info => <div className="calendar-event"><span>{info.timeText}</span><strong>{info.event.title}</strong>{info.view.type.startsWith('timeGrid') && info.event.extendedProps.description ? <p className="event-message">{info.event.extendedProps.description}</p> : <small>{info.event.extendedProps.state}</small>}</div>}
          noEventsContent="No bookings in this period" />
        {reservations.data?.length === 5000 && <p className="notice warning">Display limit reached. Narrow the resource, member, or date range.</p>}
        <div className="calendar-legend"><CalendarDays size={15}/><span>Select an available time to book</span><span className="timezone">Japan Standard Time (JST)</span></div>
      </section>
    </div>
    {editor && settings.data && <ReservationDialog key={editor.reservation?.id || 'new'} editor={editor} members={members.data || []} resources={resources.data || []} settings={settings.data} onClose={() => setEditor(null)} onSuccess={r => { setEditor(null); setNotice(r); }}/>}
  </>;
}
const colors = ['#387668','#617d9b','#ae794c','#827197','#a26972','#748648'];

function ReservationDialog({ editor, members, resources, settings, onClose, onSuccess }: {editor:Editor;members:Member[];resources:Resource[];settings:Settings;onClose:()=>void;onSuccess:(r:Result)=>void}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const old = editor.reservation;
  const client = useQueryClient();
  const [form, setForm] = useState({ resource_id:editor.resource,user_id:old?.user_id || editor.member,start:editor.start,end:editor.end,description:old?.description || '',password:'',new_password:'' });
  const [changePassword, setChangePassword] = useState(false);
  const [confirmChange, setConfirmChange] = useState<string | null>(null);
  const [confirmCancel, setConfirmCancel] = useState(false);
  const [localError, setLocalError] = useState('');
  const ended = old ? new Date(old.end).getTime() <= Date.now() : false;
  const started = old ? new Date(old.start).getTime() <= Date.now() : false;
  useEffect(() => { dialog.current?.showModal(); }, []);
  const mutation = useMutation({
    mutationFn: (remove: boolean) => {
      if (remove) return api<Result>('/reservations/' + old!.id, 'DELETE', { password:form.password,version:old!.version });
      const data = { ...form, start:old && form.start === dateInput(old.start) ? old.start : toApiDate(form.start), end:old && form.end === dateInput(old.end) ? old.end : toApiDate(form.end), new_password:changePassword ? form.new_password : undefined, version:old?.version };
      return api<Result>(old ? '/reservations/' + old.id : '/reservations', old ? 'PUT' : 'POST', data);
    },
    onSuccess: async result => { await client.invalidateQueries({ queryKey:['reservations'] }); onSuccess(result); },
  });
  const update = (key: keyof typeof form, value: string | number) => { setForm(previous => ({ ...previous,[key]:value })); setLocalError(''); setConfirmChange(null); mutation.reset(); };
  function submit(event: FormEvent) {
    event.preventDefault();
    if (!form.user_id || !form.resource_id) { setLocalError('Select a resource and member.'); return; }
    if (!form.start || !form.end || new Date(toApiDate(form.start)) >= new Date(toApiDate(form.end))) { setLocalError('End time must be after start time.'); return; }
    const warning = slackWarning(old, old && form.start === dateInput(old.start) ? old.start : toApiDate(form.start), old && form.end === dateInput(old.end) ? old.end : toApiDate(form.end));
    if (warning) setConfirmChange(warning);
    else mutation.mutate(false);
  }
  return <dialog ref={dialog} className="reservation-dialog" onCancel={event => { event.preventDefault(); if (!mutation.isPending) onClose(); }}><form onSubmit={submit}><div className="dialog-heading"><div><p className="eyebrow">{old ? 'RESERVATION DETAIL' : 'NEW RESERVATION'}</p><h2>{old ? 'Booking details' : 'Book a resource'}</h2></div><button type="button" className="icon-button" aria-label="Close" onClick={onClose} disabled={mutation.isPending}><X size={21}/></button></div>
    {old && <div className="detail-strip"><span className="badge">{status(old.start,old.end)}</span><span>{old.resource_name} · {old.user_name}</span></div>}
    {ended && <p className="notice">Completed bookings cannot be edited or cancelled.</p>}
    <fieldset disabled={ended || mutation.isPending || Boolean(confirmChange)}>
      <div className="form-grid"><label>Resource<select value={form.resource_id} onChange={e => update('resource_id', Number(e.target.value))} disabled={started} required><option value={0}>Select a resource</option>{resources.filter(r => r.active || r.id === form.resource_id).map(r => <option key={r.id} value={r.id}>{r.resource_name}{r.active ? '' : ' (inactive)'}</option>)}</select></label><label>Member<select value={form.user_id} onChange={e => update('user_id', Number(e.target.value))} required><option value={0}>Select a member</option>{members.filter(m => m.active || m.id === form.user_id).map(m => <option key={m.id} value={m.id}>{m.user_name}{m.active ? '' : ' (inactive)'}</option>)}</select></label><label>Start time<input type="datetime-local" required value={form.start} onChange={e => update('start',e.target.value)} disabled={started}/></label><label>End time<input type="datetime-local" required value={form.end} onChange={e => update('end',e.target.value)}/></label></div>
      <p className="field-hint">JST · Book up to {settings.advance_days} days ahead. Overlapping bookings are not allowed.</p>
      <label>Notes / messages <span className="optional">optional</span><textarea rows={3} maxLength={4000} value={form.description} onChange={e => update('description',e.target.value)} placeholder="Purpose, messages, or handover notes"/></label>
      <label>{old ? 'Booking password' : 'Booking password'} <span className="optional">{old ? 'leave blank if none was set' : 'optional'}</span><input type="password" autoComplete={old ? 'current-password' : 'new-password'} maxLength={256} value={form.password} onChange={e => update('password',e.target.value)}/></label>
      {!old && <p className="field-hint">Without a password, anyone with access to this site can edit or cancel the booking.</p>}
      {old && <><label className="checkbox-label"><input type="checkbox" checked={changePassword} onChange={e => setChangePassword(e.target.checked)}/>Change password</label>{changePassword && <label>New password<input type="password" maxLength={256} autoComplete="new-password" value={form.new_password} onChange={e => update('new_password',e.target.value)}/><span className="field-hint">Leave blank to remove password protection.</span></label>}</>}
    </fieldset>
    {(localError || mutation.error) && <div className="notice error" role="alert">{localError || mutation.error?.message}</div>}
    {confirmChange && <div className="notice warning slack-confirmation" role="alert"><div><strong>Slack notification required</strong><p>{confirmChange}</p>{settings.notification_text && <p>{settings.notification_text}</p>}<button type="button" className="primary" disabled={mutation.isPending} onClick={() => mutation.mutate(false)}>Confirm changes</button><button type="button" disabled={mutation.isPending} onClick={() => setConfirmChange(null)}>Go back</button></div></div>}
    {confirmCancel && <div className="notice warning"><div><strong>Cancel this booking?</strong><p>{displayDate(old!.start)} booking.</p>{slackWarning(old,old!.start,old!.end,true) && <p>{slackWarning(old,old!.start,old!.end,true)}</p>}<button type="button" className="danger" disabled={mutation.isPending} onClick={() => mutation.mutate(true)}>Confirm cancellation</button><button type="button" disabled={mutation.isPending} onClick={() => setConfirmCancel(false)}>Go back</button></div></div>}
    <div className="dialog-actions">{old && !ended && !started && <button type="button" className="danger-outline" disabled={mutation.isPending} onClick={() => setConfirmCancel(true)}>Cancel booking</button>}<span className="spacer"/><button type="button" disabled={mutation.isPending} onClick={onClose}>{ended ? 'Close' : 'Cancel'}</button>{!ended && <button type="submit" className="primary" disabled={mutation.isPending || confirmCancel || Boolean(confirmChange)}>{mutation.isPending ? 'Saving…' : old ? 'Save changes' : 'Create booking'}</button>}</div>
  </form></dialog>;
}
