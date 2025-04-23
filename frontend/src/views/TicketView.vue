<!-- TicketView.vue -->
<script setup lang="ts">
import { ref, watch, computed, onMounted } from "vue";
import { useRoute, useRouter } from "vue-router";
import { useRecentTicketsStore } from "@/stores/recentTickets";
import { useTitleManager } from '@/composables/useTitleManager';
import { useAuthStore } from '@/stores/auth';
import CollaborativeTicketArticle from '@/components/ticketComponents/CollaborativeTicketArticle.vue';
import TicketDetails from '@/components/ticketComponents/TicketDetails.vue'
import DeviceDetails from '@/components/ticketComponents/DeviceDetails.vue';
import DeviceSelectionModal from '@/components/ticketComponents/DeviceSelectionModal.vue';
import CommentsAndAttachments from "@/components/ticketComponents/CommentsAndAttachments.vue";
import LinkedTicketModal from "@/components/ticketComponents/LinkedTicketModal.vue";
import LinkedTicketPreview from "@/components/ticketComponents/LinkedTicketPreview.vue";
import ProjectSelectionModal from "@/components/ticketComponents/ProjectSelectionModal.vue";
import ProjectInfo from "@/components/ticketComponents/ProjectInfo.vue";
import UserAvatar from "@/components/UserAvatar.vue";
import { STATUS_OPTIONS, PRIORITY_OPTIONS } from '@/constants/ticketOptions';
import type { TicketStatus, TicketPriority } from '@/constants/ticketOptions';
import ticketService from '@/services/ticketService';
import userService from '@/services/userService';
import type { Ticket, Device as TicketDevice, Attachment, Project, Comment } from '@/services/ticketService';
import type { User } from '@/services/userService';
import BackButton from '@/components/common/BackButton.vue';
import DeleteButton from '@/components/common/DeleteButton.vue';
import { projectService } from '@/services/projectService';
import * as deviceService from '@/services/deviceService';
import type { Device } from '@/types/device';

// User info interface for comments
interface UserInfo {
  uuid: string;
  name: string;
}

// Custom attachment type for compatibility with existing components
interface UIAttachment {
  url: string;
  name: string;
  id: number;
  comment_id: number;
}

// Interface for comments with UI-compatible attachments
interface CommentWithAttachments {
  id: number;
  content: string;
  user_uuid: string;
  createdAt: string;
  created_at: string;
  ticket_id: number;
  attachments?: UIAttachment[];
  user?: UserInfo;
}

// Extended ticket interface with additional UI-specific properties
interface LocalTicket extends Ticket {
  commentsAndAttachments?: CommentWithAttachments[];
  project?: string;
  devices?: TicketDevice[];
}

const route = useRoute();
const router = useRouter();
const ticket = ref<LocalTicket | null>(null);
const projectDetails = ref<Project | null>(null);
const recentTicketsStore = useRecentTicketsStore();
const titleManager = useTitleManager();
const authStore = useAuthStore();

// Add users state
const users = ref<User[]>([]);
const loadingUsers = ref(true);
const usersError = ref<string | null>(null);

const selectedStatus = ref<TicketStatus>("open");
const selectedPriority = ref<TicketPriority>("low");
const showDeviceModal = ref(false);
const showProjectModal = ref(false);
const loading = ref(true);
const error = ref<string | null>(null);

// Add isNewTicket flag and editingTitle state
const isNewTicket = ref(false);
const editingTitle = ref(false);
const titleInput = ref<HTMLInputElement | null>(null);
const newTitleValue = ref('');

// Add a function to transform backend Device to frontend Device format
const transformDevice = (backendDevice: Device): TicketDevice => {
  return {
    id: backendDevice.id,
    name: backendDevice.name,
    hostname: backendDevice.hostname,
    serial_number: backendDevice.serial_number,
    model: backendDevice.model,
    warranty_status: backendDevice.warranty_status
  };
};

// Add function to fetch users from the backend
const fetchUsers = async () => {
  loadingUsers.value = true;
  usersError.value = null;
  
  try {
    users.value = await userService.getUsers();
    console.log('Users loaded:', users.value);
  } catch (err) {
    console.error('Error fetching users:', err);
    usersError.value = 'Failed to load users. Using fallback data if available.';
  } finally {
    loadingUsers.value = false;
  }
};

const fetchTicket = async (ticketId: string | string[]) => {
  const id = Number(ticketId);
  loading.value = true;
  error.value = null;
  
  console.log(`Fetching ticket #${id} from server...`);
  
  try {
    const fetchedTicket = await ticketService.getTicketById(id);
    
    if (!fetchedTicket) {
      router.push("/404");
      return;
    }

    console.log(`Received ticket data from server:`, fetchedTicket);
    console.log(`Linked tickets from server:`, fetchedTicket.linked_tickets);
    console.log(`Projects from server:`, fetchedTicket.projects);

    // Check if this is a new ticket (title is "New Ticket")
    isNewTicket.value = fetchedTicket.title === 'New Ticket';
    
    // If it's a new ticket, set the title value for editing
    if (isNewTicket.value) {
      editingTitle.value = true;
      newTitleValue.value = '';
      // Set a slight delay to ensure the DOM is ready
      setTimeout(() => {
        if (titleInput.value) {
          titleInput.value.focus();
        }
      }, 100);
    }

    // Transform comments to commentsAndAttachments format expected by components
    const commentsAndAttachments = fetchedTicket.comments?.map(comment => ({
      id: comment.id,
      content: comment.content,
      user_uuid: comment.user_uuid,
      createdAt: comment.created_at,
      created_at: comment.created_at,
      ticket_id: comment.ticket_id,
      attachments: comment.attachments?.map(att => ({
        id: att.id,
        url: att.url,
        name: att.name,
        comment_id: att.comment_id
      })) || [],
      user: comment.user // Include the user information from the backend
    })) || [];

    // Sort comments by created_at date in descending order (newest first)
    commentsAndAttachments.sort((a, b) => {
      return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
    });

    // Transform devices to match the frontend Device type
    const transformedDevices = fetchedTicket.device 
      ? [transformDevice(fetchedTicket.device)] 
      : [];

    // Update the ticket with the fetched data
    ticket.value = {
      ...fetchedTicket,
      linkedTickets: fetchedTicket.linked_tickets || fetchedTicket.linkedTickets || [],
      devices: transformedDevices,
      commentsAndAttachments
    } as unknown as LocalTicket;

    console.log(`Processed linked tickets:`, ticket.value.linkedTickets);

    // Update the selected values to match the ticket
    selectedStatus.value = ticket.value.status;
    selectedPriority.value = ticket.value.priority;

    const fromRecent = route.query.fromRecent === "true";
    
    // Check if the ticket already exists in the recent tickets store
    const existingTicket = recentTicketsStore.recentTickets.find(t => t.id === id);
    
    if (existingTicket) {
      console.log(`Ticket #${id} already exists in recent tickets store with title: "${existingTicket.title}"`);
      console.log(`Server returned title: "${fetchedTicket.title}"`);
      
      // Only update specific fields, preserving the title if it was manually changed
      recentTicketsStore.updateTicketData(id, {
        status: fetchedTicket.status,
        requester: fetchedTicket.requester,
        assignee: fetchedTicket.assignee,
        modified: fetchedTicket.modified
      });
    } else {
      // If the ticket doesn't exist in the store, add it
      console.log(`Adding ticket #${id} to recent tickets store with title: "${fetchedTicket.title}"`);
      recentTicketsStore.addRecentTicket(
        {
          id: ticket.value.id,
          title: ticket.value.title,
          status: ticket.value.status,
          requester: ticket.value.requester,
          assignee: ticket.value.assignee,
          created: ticket.value.created,
          modified: ticket.value.modified
        },
        fromRecent,
      );
    }

    // Update project details if projects are available
    if (fetchedTicket.projects && fetchedTicket.projects.length > 0) {
      // Use the project data directly from the API response
      projectDetails.value = fetchedTicket.projects[0];
      // Store the project ID in the ticket for reference
      ticket.value.project = String(fetchedTicket.projects[0].id);
    } else if (ticket.value.project) {
      await fetchProjectDetails(ticket.value.project);
    }
    
    console.log('Ticket data refreshed:', ticket.value);
  } catch (err) {
    console.error(`Error fetching ticket ${id}:`, err);
    error.value = 'Failed to load ticket. Please try again later.';
  } finally {
    loading.value = false;
  }
};

// Add a refresh function to reload the ticket data
const refreshTicket = async () => {
  if (ticket.value) {
    const ticketId = ticket.value.id;
    console.log(`Refreshing ticket #${ticketId} data...`);
    
    // Store the current title and linked tickets before refreshing
    const currentTitle = ticket.value.title;
    const currentLinkedTickets = [...(ticket.value.linkedTickets || [])];
    console.log(`Current title before refresh: "${currentTitle}"`);
    console.log(`Current linked tickets before refresh: ${JSON.stringify(currentLinkedTickets)}`);
    
    // Fetch fresh data from the server
    await fetchTicket(String(ticketId));
    
    // Check if the title was preserved
    console.log(`Title after refresh: "${ticket.value?.title}"`);
    console.log(`Linked tickets after refresh: ${JSON.stringify(ticket.value?.linkedTickets)}`);
    
    // Ensure the recent tickets store has the latest title
    if (ticket.value && ticket.value.title !== currentTitle) {
      console.log(`Title changed during refresh. Updating recent tickets store with new title: "${ticket.value.title}"`);
      recentTicketsStore.updateTicketData(ticketId, {
        title: ticket.value.title
      });
    }
  }
};

// Format users for the UserSelection component
const formattedUsers = computed(() => {
  return users.value.map(user => ({
    id: user.uuid,
    name: user.name,
    email: user.email
  }));
});

const formattedDate = (dateString: string | undefined) => {
  if (!dateString) return "";
  const date = new Date(dateString);
  return date.toLocaleDateString("en-US", {
    year: "numeric",
    month: "long",
    day: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
};

const formattedCreatedDate = computed(() => formattedDate(ticket.value?.created));
const formattedModifiedDate = computed(() => formattedDate(ticket.value?.modified));

// Fetch users and ticket data when component mounts
onMounted(async () => {
  await fetchUsers();
  if (route.params.id) {
    fetchTicket(route.params.id);
  }
});

watch(
  () => route.params.id,
  (newId) => {
    if (newId) {
      fetchTicket(newId);
    }
  }
);

// Helper function to get current UTC datetime formatted for the backend
const getCurrentUTCDateTime = () => {
  const now = new Date();
  return now.toISOString(); // Return full ISO datetime string
};

const updateStatus = async (newStatus: string) => {
  // Convert string to TicketStatus type
  const typedStatus = newStatus as TicketStatus;
  selectedStatus.value = typedStatus;
  
  if (ticket.value) {
    try {
      // Update the local state
      ticket.value.status = typedStatus;
      // Get current UTC datetime
      const nowDateTime = getCurrentUTCDateTime();
      ticket.value.modified = nowDateTime;
      
      // Send the update to the backend
      await ticketService.updateTicket(ticket.value.id, { 
        status: typedStatus,
        modified: nowDateTime
      });
      
      // Update the ticket in the recent tickets store
      recentTicketsStore.updateTicketData(ticket.value.id, {
        status: typedStatus
      });
      
      // Refresh the ticket data to ensure it's up to date
      await refreshTicket();
    } catch (err) {
      console.error(`Error updating ticket status:`, err);
      // Revert UI if update fails
      ticket.value.status = selectedStatus.value;
    }
  }
};

const updatePriority = async (newPriority: string) => {
  // Convert string to TicketPriority type
  const typedPriority = newPriority as TicketPriority;
  selectedPriority.value = typedPriority;
  
  if (ticket.value) {
    try {
      // Update the local state
      ticket.value.priority = typedPriority;
      // Get current UTC datetime
      const nowDateTime = getCurrentUTCDateTime();
      ticket.value.modified = nowDateTime;
      
      // Send the update to the backend
      await ticketService.updateTicket(ticket.value.id, { 
        priority: typedPriority,
        modified: nowDateTime
      });
      
      // Update the ticket in the recent tickets store
      recentTicketsStore.updateTicketData(ticket.value.id, {
        // Priority is not stored in the recent tickets, but we'll update modified date
        modified: nowDateTime
      });
      
      // Refresh the ticket data to ensure it's up to date
      await refreshTicket();
    } catch (err) {
      console.error(`Error updating ticket priority:`, err);
      // Revert UI if update fails
      ticket.value.priority = selectedPriority.value;
    }
  }
};

// Add handlers for requester and assignee updates
const updateRequester = async (newRequester: string) => {
  if (ticket.value) {
    const oldRequester = ticket.value.requester;
    try {
      console.log(`Updating requester from ${oldRequester} to ${newRequester}`);
      
      // Update the local state
      ticket.value.requester = newRequester;
      // Get current UTC datetime
      const nowDateTime = getCurrentUTCDateTime();
      ticket.value.modified = nowDateTime;
      
      // Send the update to the backend
      const updateData = { 
        requester: newRequester,
        modified: nowDateTime
      };
      console.log('Sending update to backend:', updateData);
      
      await ticketService.updateTicket(ticket.value.id, updateData);
      console.log(`Requester updated to: ${newRequester}`);
      
      // Update the ticket in the recent tickets store
      recentTicketsStore.updateTicketData(ticket.value.id, {
        requester: newRequester,
        modified: nowDateTime
      });
      
      // Refresh the ticket data to ensure it's up to date
      await refreshTicket();
    } catch (err) {
      console.error(`Error updating ticket requester:`, err);
      // Revert UI if update fails
      ticket.value.requester = oldRequester;
    }
  }
};

const updateAssignee = async (newAssignee: string) => {
  if (ticket.value) {
    const oldAssignee = ticket.value.assignee;
    try {
      console.log(`Updating assignee from ${oldAssignee} to ${newAssignee}`);
      
      // Update the local state
      ticket.value.assignee = newAssignee;
      // Get current UTC datetime
      const nowDateTime = getCurrentUTCDateTime();
      ticket.value.modified = nowDateTime;
      
      // Send the update to the backend
      const updateData = { 
        assignee: newAssignee,
        modified: nowDateTime
      };
      console.log('Sending update to backend:', updateData);
      
      await ticketService.updateTicket(ticket.value.id, updateData);
      console.log(`Assignee updated to: ${newAssignee}`);
      
      // Update the ticket in the recent tickets store
      recentTicketsStore.updateTicketData(ticket.value.id, {
        assignee: newAssignee,
        modified: nowDateTime
      });
      
      // Refresh the ticket data to ensure it's up to date
      await refreshTicket();
    } catch (err) {
      console.error(`Error updating ticket assignee:`, err);
      // Revert UI if update fails
      ticket.value.assignee = oldAssignee;
    }
  }
};

const emit = defineEmits<{
  (e: 'update:ticket', ticket: { id: number; title: string } | null): void;
}>();

// Watch ticket changes to emit updates
watch(ticket, (newTicket) => {
  if (newTicket) {
    emit('update:ticket', {
      id: newTicket.id,
      title: newTicket.title
    });
    titleManager.setCustomTitle(`#${newTicket.id} ${newTicket.title}`);
  } else {
    emit('update:ticket', null);
  }
}, { immediate: true });

const updateTicketTitle = async (newTitle: string) => {
  if (ticket.value) {
    const oldTitle = ticket.value.title;
    const ticketId = ticket.value.id;
    
    console.log(`Updating ticket #${ticketId} title from "${oldTitle}" to "${newTitle}"`);
    
    try {
      // Update the local state
      ticket.value.title = newTitle;
      
      // Get current UTC datetime
      const nowDateTime = getCurrentUTCDateTime();
      ticket.value.modified = nowDateTime;
      
      // Update the title in the UI
      titleManager.setCustomTitle(`#${ticket.value.id} ${newTitle}`);
      
      // Update the ticket in the recent tickets store BEFORE the API call
      console.log('Updating title in recent tickets store...');
      recentTicketsStore.updateTicketData(ticketId, {
        title: newTitle,
        modified: nowDateTime
      });
      
      // Send the update to the backend
      console.log('Sending title update to backend...');
      await ticketService.updateTicket(ticketId, { 
        title: newTitle,
        modified: nowDateTime
      });
      
      console.log('Title update successful');
      
      // Refresh the ticket data to ensure it's up to date
      await refreshTicket();
    } catch (err) {
      console.error(`Error updating ticket title:`, err);
      // Revert UI if update fails
      ticket.value.title = oldTitle;
      titleManager.setCustomTitle(`#${ticket.value.id} ${oldTitle}`);
      
      // Also revert the title in the recent tickets store
      recentTicketsStore.updateTicketData(ticketId, {
        title: oldTitle
      });
    }
  }
};

const handleAddDevice = async (device: Device) => {
  if (ticket.value) {
    if (!ticket.value.devices) {
      ticket.value.devices = [];
    }
    
    try {
      console.log(`Adding device ${device.id} to ticket ${ticket.value.id}`);
      
      // Create a new object with the correct types for the API
      const deviceUpdate = {
        name: device.name,
        hostname: device.hostname,
        serial_number: device.serial_number,
        model: device.model,
        warranty_status: device.warranty_status,
        ticket_id: ticket.value.id
      };
      
      // Update the device's ticket_id
      const updatedDevice = await deviceService.updateDevice(device.id, deviceUpdate);
      
      // Add the device to the local state
      ticket.value.devices.push(transformDevice(updatedDevice));
      showDeviceModal.value = false;
      
      // Refresh the ticket data to ensure it's up to date
      await refreshTicket();
      
      console.log(`Successfully added device ${device.id} to ticket ${ticket.value.id}`);
    } catch (err) {
      console.error(`Error adding device to ticket:`, err);
      // Revert UI if update fails
      if (ticket.value.devices) {
        ticket.value.devices = ticket.value.devices.filter(d => d.id !== device.id);
      }
    }
  }
};

const removeDevice = async (deviceId: number) => {
  if (ticket.value && ticket.value.devices) {
    const originalDevices = [...ticket.value.devices];
    try {
      console.log(`Removing device ${deviceId} from ticket ${ticket.value.id}`);
      
      // Find the device to remove
      const deviceToRemove = ticket.value.devices.find(d => d.id === deviceId);
      if (!deviceToRemove) {
        console.error(`Device ${deviceId} not found in ticket ${ticket.value.id}`);
        return;
      }
      
      // Create a new object with the correct types for the API
      const deviceUpdate = {
        name: deviceToRemove.name,
        hostname: deviceToRemove.hostname,
        serial_number: deviceToRemove.serial_number,
        model: deviceToRemove.model,
        warranty_status: deviceToRemove.warranty_status,
        ticket_id: null
      };
      
      // Update the device to remove the ticket_id
      await deviceService.updateDevice(deviceId, deviceUpdate);
      
      // Remove the device from the local state
      ticket.value.devices = ticket.value.devices.filter(d => d.id !== deviceId);
      
      // Refresh the ticket data to ensure it's up to date
      await refreshTicket();
      
      console.log(`Successfully removed device ${deviceId} from ticket ${ticket.value.id}`);
    } catch (err) {
      console.error(`Error removing device from ticket:`, err);
      // Revert UI if update fails
      ticket.value.devices = originalDevices;
    }
  }
};

const showLinkedTicketModal = ref(false);

const handleLinkTicket = async (linkedTicketId: number) => {
  if (ticket.value) {
    if (!ticket.value.linkedTickets) {
      ticket.value.linkedTickets = [];
    }
    
    if (!ticket.value.linkedTickets.includes(linkedTicketId)) {
      try {
        console.log(`Attempting to link ticket ${ticket.value.id} to ticket ${linkedTicketId}`);
        
        // Call the API to link the tickets
        await ticketService.linkTicket(ticket.value.id, linkedTicketId);
        console.log(`API call successful for linking ticket ${ticket.value.id} to ${linkedTicketId}`);
        
        // Don't update the local state here, instead refresh the ticket data
        // to get the updated linked_tickets from the server
        await refreshTicket();
        
        console.log(`Successfully linked ticket ${ticket.value.id} to ticket ${linkedTicketId}`);
        console.log(`Updated linked tickets: ${JSON.stringify(ticket.value.linkedTickets)}`);
      } catch (err) {
        console.error(`Error linking ticket:`, err);
      }
    } else {
      console.log(`Ticket ${linkedTicketId} is already linked to ticket ${ticket.value.id}`);
    }
  }
};

const unlinkTicket = async (linkedTicketId: number) => {
  if (ticket.value) {
    try {
      console.log(`Attempting to unlink ticket ${ticket.value.id} from ticket ${linkedTicketId}`);
      
      // Call the API to unlink the tickets
      await ticketService.unlinkTicket(ticket.value.id, linkedTicketId);
      console.log(`API call successful for unlinking ticket ${ticket.value.id} from ${linkedTicketId}`);
      
      // Don't update local state here, instead refresh the ticket data
      // to get the updated linked_tickets from the server
      await refreshTicket();
      
      console.log(`Successfully unlinked ticket ${ticket.value.id} from ticket ${linkedTicketId}`);
      console.log(`Updated linked tickets: ${JSON.stringify(ticket.value.linkedTickets)}`);
    } catch (err) {
      console.error(`Error unlinking ticket:`, err);
    }
  }
};

const handleAddToProject = async (projectId: number) => {
  if (ticket.value) {
    try {
      // Add the ticket to the project
      await projectService.addTicketToProject(projectId, ticket.value.id);
      
      // Fetch the project details
      const project = await projectService.getProject(projectId);
      
      // Convert the project to the format expected by the API
      projectDetails.value = {
        id: project.id,
        name: project.name,
        description: project.description,
        status: project.status,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
        ticket_count: project.ticketCount || 0 // Include ticket count if available
      };
      
      ticket.value.project = String(projectId);
      showProjectModal.value = false;
      
      console.log(`Added ticket ${ticket.value.id} to project ${projectId}`);
      
      // Refresh the ticket to get the updated project information
      await refreshTicket();
    } catch (err) {
      console.error(`Error adding ticket to project:`, err);
      // You could show an error notification here
    }
  }
}

// Update fetchProjectDetails function
const fetchProjectDetails = async (projectId: string) => {
  try {
    const project = await projectService.getProject(Number(projectId));
    
    // Convert the project to the format expected by the API
    projectDetails.value = {
      id: project.id,
      name: project.name,
      description: project.description,
      status: project.status,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
      ticket_count: project.ticketCount || 0 // Include ticket count if available
    };
  } catch (err) {
    console.error(`Error fetching project details:`, err);
    // You could show an error notification here
  }
}

// Add this function to handle removing a ticket from a project
const removeFromProject = async () => {
  if (ticket.value && ticket.value.project) {
    try {
      const projectId = Number(ticket.value.project);
      
      await projectService.removeTicketFromProject(projectId, ticket.value.id);
      
      ticket.value.project = undefined;
      projectDetails.value = null;
      
      console.log(`Removed ticket ${ticket.value.id} from project ${projectId}`);
    } catch (err) {
      console.error(`Error removing ticket from project:`, err);
      // You could show an error notification here
    }
  }
}

// Add this function to handle project navigation
const viewProject = (projectId: string) => {
  router.push(`/projects/${projectId}`)
}

const getStatusColor = (status: string) => {
  switch (status) {
    case 'active':
      return 'text-green-400'
    case 'completed':
      return 'text-blue-400'
    case 'archived':
      return 'text-gray-400'
    default:
      return 'text-slate-400'
  }
}

// Add function to handle new comments
const handleAddComment = async (data: { content: string; user_uuid: string; attachments: { url: string; name: string; id?: number }[] }) => {
  if (!ticket.value) return;
  
  try {
    // Validate that we have either content or attachments
    if (!data.content.trim() && (!data.attachments || data.attachments.length === 0)) {
      console.log('Comment must have either content or attachments');
      return;
    }
    
    // If content is empty but we have attachments, set a placeholder content
    if (!data.content.trim() && data.attachments && data.attachments.length > 0) {
      data.content = "Attachment added";
    }
    
    console.log("Sending comment data to API:", data);
    
    // Create the comment with attachments
    const newComment = await ticketService.addCommentToTicket(
      ticket.value.id,
      data.content,
      data.user_uuid,
      data.attachments
    );
    
    console.log('Comment created successfully, response:', newComment);
    
    // Add the new comment to the local state immediately
    if (ticket.value && ticket.value.commentsAndAttachments) {
      // Covert the new comment to the expected format
      const commentToAdd: CommentWithAttachments = {
        id: newComment.id,
        content: newComment.content,
        user_uuid: newComment.user_uuid,
        createdAt: newComment.created_at,
        created_at: newComment.created_at,
        ticket_id: newComment.ticket_id,
        attachments: newComment.attachments,
        user: newComment.user
      };
      
      // Add it to the beginning of the array (newest first)
      ticket.value.commentsAndAttachments.unshift(commentToAdd);
      console.log('Added new comment to local state:', commentToAdd);
    } else {
      console.warn('Unable to add comment to local state, refreshing ticket data');
      // If we can't add to local state for some reason, refresh the whole ticket
      await refreshTicket();
    }
  } catch (error) {
    console.error("Error adding comment:", error);
    // If there's an error, refresh the ticket data
    await refreshTicket();
  }
};

// Add function to handle ticket deletion
const deleteTicket = async () => {
  if (!ticket.value) return;
  
  const ticketId = ticket.value.id;
  console.log(`Deleting ticket #${ticketId}...`);
  
  try {
    // Call the API to delete the ticket
    await ticketService.deleteTicket(ticketId);
    
    // Remove the ticket from the recent tickets store
    recentTicketsStore.removeRecentTicket(ticketId);
    
    console.log(`Ticket #${ticketId} deleted successfully`);
    
    // Navigate back to the tickets list
    router.push('/tickets');
  } catch (err) {
    console.error(`Error deleting ticket #${ticketId}:`, err);
    // You could show an error notification here
  }
};

// Add function to handle deleting attachments
const handleDeleteAttachment = async (data: { commentId: number; attachmentIndex: number }) => {
  if (!ticket.value) return;
  
  try {
    console.log('Deleting attachment:', data);
    
    // Get the comment and attachment
    const comment = ticket.value.commentsAndAttachments?.find(c => c.id === data.commentId);
    if (!comment || !comment.attachments || comment.attachments.length <= data.attachmentIndex) {
      console.error('Attachment not found');
      return;
    }
    
    const attachment = comment.attachments[data.attachmentIndex];
    console.log('Attachment details:', attachment);
    
    // Check if the attachment has an ID (from the backend)
    if (attachment.id) {
      // Delete the attachment from the backend
      await ticketService.deleteAttachment(attachment.id);
      
      console.log('Attachment deleted successfully');
      
      // Check if this is the last attachment and the comment is empty
      if (comment.attachments.length === 1 && (!comment.content || comment.content.trim() === '')) {
        console.log('Last attachment deleted from empty comment. Deleting comment:', comment.id);
        await handleDeleteComment(comment.id);
      } else {
        // Just refresh the ticket data to ensure it's up to date
        await refreshTicket();
      }
    } else {
      console.error('Cannot delete attachment without ID. Attachment:', attachment);
    }
  } catch (err) {
    console.error('Error deleting attachment:', err);
    // You could show an error notification here
  }
};

// Add function to handle deleting comments
const handleDeleteComment = async (commentId: number) => {
  if (!ticket.value) return;
  
  try {
    console.log('Deleting comment:', commentId);
    
    // Delete the comment from the backend
    await ticketService.deleteComment(commentId);
    
    console.log('Comment deleted successfully');
    
    // Refresh the ticket data to ensure it's up to date
    await refreshTicket();
  } catch (err) {
    console.error('Error deleting comment:', err);
    // You could show an error notification here
  }
};

const navigateToDeviceView = (deviceId: number) => {
  router.push({
    path: `/devices/${deviceId}`,
    query: { fromTicket: String(ticket.value?.id) }
  });
};
</script>

<template>
  <div class="flex-1">
    <div v-if="ticket" class="flex flex-col">
      <!-- Navigation and actions bar -->
      <div class="pt-4 px-6 flex justify-between items-center">
        <BackButton 
          v-if="ticket.project" 
          context="project" 
          :contextId="ticket.project" 
          :fallbackRoute="'/tickets'" 
        />
        <BackButton 
          v-else 
          fallbackRoute="/tickets" 
        />
        <DeleteButton 
          fallbackRoute="/tickets" 
          itemName="Ticket" 
          @delete="deleteTicket" 
        />
      </div>
      
      <div class="flex flex-col gap-4 px-6 py-4 mx-auto w-full max-w-8xl">
        <!-- Grid Container -->
        <div class="grid-container">
          <!-- Details (TicketDetails and DeviceDetails) -->
          <div class="details-area flex flex-col gap-4">
            <TicketDetails 
              :ticket="ticket" 
              :created-date="formattedCreatedDate" 
              :modified-date="formattedModifiedDate"
              :selected-status="selectedStatus" 
              :selected-priority="selectedPriority"
              :status-options="STATUS_OPTIONS" 
              :priority-options="PRIORITY_OPTIONS"
              :users="formattedUsers"
              @update:selectedStatus="updateStatus" 
              @update:selectedPriority="updatePriority"
              @update:requester="updateRequester"
              @update:assignee="updateAssignee"
            />
            
            <!-- Device section -->
            <template v-if="ticket">
              <div v-if="ticket.devices?.length" class="flex flex-col gap-2">
                <div class="flex items-center justify-between">
                  <h3 class="text-sm font-medium text-slate-300">Devices</h3>
                  <a 
                    href="#" 
                    @click.prevent="showDeviceModal = true" 
                    class="text-blue-500 hover:text-blue-400 text-sm hover:underline"
                  >
                    + Add device
                  </a>
                </div>
                <div class="flex flex-col gap-2">
                  <DeviceDetails
                    v-for="device in ticket.devices"
                    :key="device.id"
                    :id="device.id"
                    :name="device.name"
                    :hostname="device.hostname"
                    :serial_number="device.serial_number"
                    :model="device.model"
                    :warranty_status="device.warranty_status"
                    @remove="() => removeDevice(device.id)"
                    @view="navigateToDeviceView"
                  />
                </div>
              </div>
              <div v-else>
                <a href="#" @click.prevent="showDeviceModal = true" class="block text-blue-500 hover:underline">+ Add device</a>
              </div>

              <!-- Add DeviceModal component -->
              <DeviceSelectionModal
                :show="showDeviceModal"
                :current-ticket-id="ticket?.id"
                :existing-device-ids="ticket?.devices?.map(d => d.id) || []"
                @close="showDeviceModal = false"
                @select-device="handleAddDevice"
              />

              <!-- Linked ticket section -->
              <div v-if="ticket.linkedTickets?.length" class="flex flex-col gap-2">
                <div class="flex items-center justify-between">
                  <h3 class="text-sm font-medium text-slate-300">Linked Tickets</h3>
                  <a 
                    href="#" 
                    @click.prevent="showLinkedTicketModal = true" 
                    class="text-blue-500 hover:text-blue-400 text-sm hover:underline"
                  >
                    + Add linked ticket
                  </a>
                </div>
                <div class="flex flex-col gap-2">
                  <LinkedTicketPreview
                    v-for="linkedId in ticket.linkedTickets"
                    :key="linkedId"
                    :linked-ticket-id="linkedId"
                    :current-ticket-id="ticket.id"
                    @unlink="() => unlinkTicket(linkedId)"
                    @view="() => {}"
                  />
                </div>
              </div>
              <div v-else>
                <a href="#" @click.prevent="showLinkedTicketModal = true" class="block text-blue-500 hover:underline">+ Add linked ticket</a>
              </div>

              <!-- Project section -->
              <div class="flex flex-col gap-2">
                <div v-if="ticket.project" class="flex items-center justify-between">
                  <h3 class="text-sm font-medium text-slate-300">Project</h3>
                  <a 
                    href="#" 
                    @click.prevent="showProjectModal = true" 
                    class="text-blue-500 hover:text-blue-400 text-sm hover:underline"
                  >
                    Change project
                  </a>
                </div>

                <div v-if="ticket.project && projectDetails">
                  <!-- Project Info -->
                  <ProjectInfo
                    :project="projectDetails"
                    :project-id="ticket.project"
                    @view="viewProject(ticket.project!)"
                    @remove="removeFromProject"
                  />
                </div>

                <div v-else>
                  <a 
                    href="#" 
                    @click.prevent="showProjectModal = true" 
                    class="block text-blue-500 hover:underline"
                  >
                    + Add to project
                  </a>
                </div>

                <!-- Project Selection Modal -->
                <ProjectSelectionModal
                  :show="showProjectModal"
                  :current-project-id="ticket.project ? Number(ticket.project) : undefined"
                  @close="showProjectModal = false"
                  @select-project="handleAddToProject"
                />
              </div>
            </template>
          </div>

          <!-- Collaborative Ticket Article Body -->
          <div class="article-area">
            <CollaborativeTicketArticle 
              :initial-content="ticket.article_content || ''"
              :ticket-id="ticket.id"
            />
          </div>

          <!-- Comments and Attachments -->
          <div class="comments-area">
            <CommentsAndAttachments 
              :comments="ticket?.commentsAndAttachments || []"
              :current-user="authStore.user?.uuid || 'Unknown User'"
              @add-comment="handleAddComment"
              @delete-attachment="handleDeleteAttachment"
              @delete-comment="handleDeleteComment"
            />
          </div>
        </div>
      </div>
    </div>

    <div v-else class="p-6 text-center text-gray-400">
      Loading ticket...
    </div>

    <LinkedTicketModal 
      v-if="ticket"
      :show="showLinkedTicketModal"
      :current-ticket-id="ticket.id"
      :existing-linked-tickets="ticket.linkedTickets"
      @close="showLinkedTicketModal = false"
      @select-ticket="handleLinkTicket"
    />
  </div>
</template>

<style scoped>
.grid-container {
  display: grid;
  grid-template-columns: 1fr;
  grid-template-rows: auto;
  grid-template-areas: "details" "article" "comments";
  gap: 1rem;

  @media (min-width: 1024px) {
    grid-template-columns: repeat(2, 1fr);
    grid-template-areas:
      "details article"
      "details comments";
  }

  @media (min-width: 1536px) {
    grid-template-columns: repeat(3, 1fr);
    grid-template-areas: "details article comments";
  }
}

.details-area {
  grid-area: details;
}

.article-area {
  grid-area: article;
}

.comments-area {
  grid-area: comments;
}
</style>