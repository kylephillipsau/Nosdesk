import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions'

export interface Device {
  id: number;
  name: string;
  hostname: string;
  serial_number: string;
  model: string;
  warranty_status: string;
  ticket_id?: number | null;
}

export interface Ticket {
  id: number;
  title: string;
  status: TicketStatus;
  priority: TicketPriority;
  created: string;
  modified: string;
  assignee: string;
  requester: string;
  devices?: Device[];
  linkedTickets: number[];
  project?: string;
  notesAndComments?: {
    id: number;
    content: string;
    author: string;
    createdAt: string;
    attachments?: { url: string; name: string }[];
  }[];
  articleContent?: string;
}
