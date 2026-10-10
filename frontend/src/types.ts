export interface Member { id: number; user_name: string; active: boolean; sort_order: number }
export interface Resource { id: number; resource_name: string; active: boolean; capacity: number; custom_field: string; sort_order: number; slack_url: string; slack_channel_id: string }
export interface Reservation { id: number; resource_id: number; resource_name: string; user_id: number; user_name: string; start: string; end: string; description: string | null; created_at: string; version: number }
export interface Settings { lab_name: string; advance_days: number; slot_minutes: number; contact_text: string; notification_text: string }
export interface Result { id: number; message: string; notification_required: boolean; notification_reason: string | null; slack_url: string | null }
export interface Maintenance { version: string; basic_auth_enabled: boolean; public_mode: string; manual_notification_count: number; backup_command: string; backup_status: { status: string; completed_at: string } | null }
