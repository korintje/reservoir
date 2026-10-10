import { NavLink, Routes, Route } from 'react-router-dom';
import { CalendarDays, Settings2 } from 'lucide-react';
import ReservationPage from './ReservationPage';
import SettingsPage from './SettingsPage';
export default function App() {
  return <><header className="app-header"><nav aria-label="Main navigation"><NavLink to="/" end><CalendarDays size={18}/>Bookings</NavLink><NavLink to="/settings"><Settings2 size={18}/>Settings</NavLink></nav></header><main><Routes><Route path="/" element={<ReservationPage/>}/><Route path="/settings" element={<SettingsPage/>}/><Route path="*" element={<p>Page not found. Return to Bookings.</p>}/></Routes></main></>;
}
