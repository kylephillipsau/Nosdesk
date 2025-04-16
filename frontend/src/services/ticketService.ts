import axios from 'axios';
import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions';

// Define the API base URL - use relative URL to work from any device
const API_BASE_URL = '/api';

// Define interfaces for our data models
export interface Device {
  id: number;
  name: string;
  hostname: string;
  serial_number: string;
  model: string;
  warranty_status: string;
  ticket_id?: number | null;
}

export interface Comment {
  id: number;
  content: string;
  user_uuid: string;
  created_at: string;
  ticket_id: number;
  attachments?: Attachment[];
  user?: UserInfo;
}

export interface Attachment {
  id: number;
  url: string;
  name: string;
  comment_id: number;
}

export interface Project {
  id: number;
  name: string;
  description?: string | null;
  status: 'active' | 'completed' | 'archived';
  created_at: string;
  updated_at: string;
  ticket_count?: number;
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
  closed_at?: string;
  device?: Device;
  comments?: Comment[];
  article_content?: string;
  linkedTickets?: number[];
  linked_tickets?: number[];
  projects?: Project[];
}

// Pagination interface
export interface PaginationParams {
  page: number;
  pageSize: number;
  sortField?: string;
  sortDirection?: 'asc' | 'desc';
  search?: string;
  status?: string;
  priority?: string;
}

// Paginated response interface
export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  pageSize: number;
  totalPages: number;
}

// User info interface for comments
export interface UserInfo {
  uuid: string;
  name: string;
}

// Add CommentWithAttachments interface
export interface CommentWithAttachments {
  comment: Comment;
  attachments: Attachment[];
  user?: UserInfo;
}

// API functions for tickets
export const getTickets = async (): Promise<Ticket[]> => {
  try {
    const response = await axios.get(`${API_BASE_URL}/tickets`);
    return response.data;
  } catch (error) {
    console.error('Error fetching tickets:', error);
    throw error;
  }
};

// Get paginated tickets
export const getPaginatedTickets = async (params: PaginationParams): Promise<PaginatedResponse<Ticket>> => {
  try {
    const response = await axios.get(`${API_BASE_URL}/tickets/paginated`, { params });
    return response.data;
  } catch (error) {
    console.error('Error fetching paginated tickets:', error);
    throw error;
  }
};

export const getTicketById = async (id: number): Promise<Ticket> => {
  try {
    const response = await axios.get(`${API_BASE_URL}/tickets/${id}`);
    return response.data;
  } catch (error) {
    console.error(`Error fetching ticket ${id}:`, error);
    throw error;
  }
};

// Remove this function as we are using the createEmptyTicket function instead
export const createTicket = async (ticket: Omit<Ticket, 'id' | 'created' | 'modified'>): Promise<Ticket> => {
  try {
    const response = await axios.post(`${API_BASE_URL}/tickets`, ticket);
    return response.data;
  } catch (error) {
    console.error('Error creating ticket:', error);
    throw error;
  }
};

export const updateTicket = async (id: number, ticket: Partial<Ticket>): Promise<Ticket> => {
  try {
    const response = await axios.patch(`${API_BASE_URL}/tickets/${id}`, ticket);
    return response.data;
  } catch (error) {
    console.error(`Error updating ticket ${id}:`, error);
    throw error;
  }
};

export const deleteTicket = async (id: number): Promise<void> => {
  try {
    await axios.delete(`${API_BASE_URL}/tickets/${id}`);
  } catch (error) {
    console.error(`Error deleting ticket ${id}:`, error);
    throw error;
  }
};

export const createEmptyTicket = async (): Promise<Ticket> => {
  console.log('createEmptyTicket called');
  try {
    console.log('Sending POST request to create empty ticket');
    const response = await axios.post(`${API_BASE_URL}/tickets/empty`);
    console.log('Empty ticket created successfully:', response.data);
    return response.data;
  } catch (error) {
    console.error('Error creating empty ticket:', error);
    throw error;
  }
};

// Link a ticket to another ticket
export const linkTicket = async (ticketId: number, linkedTicketId: number): Promise<void> => {
  try {
    await axios.post(`${API_BASE_URL}/tickets/${ticketId}/link/${linkedTicketId}`);
  } catch (error) {
    console.error(`Error linking ticket ${ticketId} to ${linkedTicketId}:`, error);
    throw error;
  }
};

// Unlink a ticket from another ticket
export const unlinkTicket = async (ticketId: number, linkedTicketId: number): Promise<void> => {
  try {
    await axios.delete(`${API_BASE_URL}/tickets/${ticketId}/unlink/${linkedTicketId}`);
  } catch (error) {
    console.error(`Error unlinking ticket ${ticketId} from ${linkedTicketId}:`, error);
    throw error;
  }
};

// Add a comment to a ticket
export const addCommentToTicket = async (
  ticketId: number, 
  content: string, 
  userUuid: string,
  attachments: { url: string; name: string }[] = []
): Promise<Comment> => {
  try {
    const response = await axios.post(`${API_BASE_URL}/tickets/${ticketId}/comments`, {
      content,
      user_uuid: userUuid,
      attachments
    });
    return response.data;
  } catch (error) {
    console.error(`Error adding comment to ticket ${ticketId}:`, error);
    throw error;
  }
};

// Add an attachment to a comment
export const addAttachmentToComment = async (commentId: number, url: string, name: string): Promise<Attachment> => {
  try {
    const response = await axios.post(`${API_BASE_URL}/comments/${commentId}/attachments`, {
      url,
      name,
    });
    return response.data;
  } catch (error) {
    console.error(`Error adding attachment to comment ${commentId}:`, error);
    throw error;
  }
};

// Delete a comment
export const deleteComment = async (commentId: number): Promise<void> => {
  try {
    await axios.delete(`${API_BASE_URL}/comments/${commentId}`);
  } catch (error) {
    console.error(`Error deleting comment ${commentId}:`, error);
    throw error;
  }
};

// Delete an attachment
export const deleteAttachment = async (attachmentId: number): Promise<void> => {
  try {
    await axios.delete(`${API_BASE_URL}/attachments/${attachmentId}`);
  } catch (error) {
    console.error(`Error deleting attachment ${attachmentId}:`, error);
    throw error;
  }
};

// Get comments for a ticket
export const getCommentsByTicketId = async (ticketId: number): Promise<CommentWithAttachments[]> => {
  try {
    const response = await axios.get(`${API_BASE_URL}/tickets/${ticketId}/comments`);
    return response.data;
  } catch (error) {
    console.error(`Error getting comments for ticket ${ticketId}:`, error);
    throw error;
  }
};

// Export default object with all functions
export default {
  getTickets,
  getPaginatedTickets,
  getTicketById,
  createTicket,
  updateTicket,
  deleteTicket,
  createEmptyTicket,
  linkTicket,
  unlinkTicket,
  addCommentToTicket,
  addAttachmentToComment,
  deleteComment,
  deleteAttachment,
  getCommentsByTicketId
}; 