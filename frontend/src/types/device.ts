export interface Device {
  id: number;
  name: string;
  hostname: string;
  serial_number: string;
  model: string;
  warranty_status: string;
  // Backend-specific field, not displayed in UI
  ticket_id?: number | null;
  // Frontend-specific fields
  type?: string;
  lastSeen?: string;
  status?: string;
  specs?: {
    cpu?: string;
    memory?: string;
    storage?: string;
    os?: string;
  };
  assignedTo?: string | null;
}

export interface DeviceFormData {
  name: string;
  hostname: string;
  serial_number: string;
  model: string;
  warranty_status: string;
  type: string;
  // ticket_id is not included in the form data
} 