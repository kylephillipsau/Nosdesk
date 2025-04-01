import axios from 'axios';
import type { Device, DeviceFormData } from '@/types/device';

// Define the API base URL
const API_BASE_URL = '/api';

/**
 * Get all devices
 * @returns Promise<Device[]> - A promise that resolves to an array of devices
 */
export const getDevices = async (): Promise<Device[]> => {
  try {
    const response = await axios.get(`${API_BASE_URL}/devices`);
    
    // Transform the backend data to match our frontend Device interface
    return response.data.map((device: any) => ({
      id: device.id,
      name: device.name,
      hostname: device.hostname,
      serial_number: device.serial_number,
      model: device.model,
      warranty_status: device.warranty_status,
      ticket_id: device.ticket_id,
      // Add frontend-specific fields with default values
      type: determineDeviceType(device.model),
      lastSeen: new Date().toISOString(),
      status: 'online',
      assignedTo: null
    }));
  } catch (error) {
    console.error('Error fetching devices:', error);
    throw error;
  }
};

/**
 * Get a device by ID
 * @param id - The ID of the device to fetch
 * @returns Promise<Device> - A promise that resolves to a device
 */
export const getDeviceById = async (id: number | string): Promise<Device> => {
  try {
    const response = await axios.get(`${API_BASE_URL}/devices/${id}`);
    const device = response.data;
    
    // Transform the backend data to match our frontend Device interface
    return {
      id: device.id,
      name: device.name,
      hostname: device.hostname,
      serial_number: device.serial_number,
      model: device.model,
      warranty_status: device.warranty_status,
      ticket_id: device.ticket_id,
      // Add frontend-specific fields with default values
      type: determineDeviceType(device.model),
      lastSeen: new Date().toISOString(),
      status: 'online',
      assignedTo: null,
      specs: getDeviceSpecs(device.model)
    };
  } catch (error) {
    console.error(`Error fetching device with ID ${id}:`, error);
    throw error;
  }
};

/**
 * Get devices by ticket ID
 * @param ticketId - The ID of the ticket
 * @returns Promise<Device | null> - A promise that resolves to a device or null
 */
export const getDeviceByTicketId = async (ticketId: number): Promise<Device | null> => {
  try {
    const response = await axios.get(`${API_BASE_URL}/tickets/${ticketId}/device`);
    const device = response.data;
    
    // Transform the backend data to match our frontend Device interface
    return {
      id: device.id,
      name: device.name,
      hostname: device.hostname,
      serial_number: device.serial_number,
      model: device.model,
      warranty_status: device.warranty_status,
      ticket_id: device.ticket_id,
      // Add frontend-specific fields with default values
      type: determineDeviceType(device.model),
      lastSeen: new Date().toISOString(),
      status: 'online',
      assignedTo: null,
      specs: getDeviceSpecs(device.model)
    };
  } catch (error) {
    console.error(`Error fetching device for ticket ID ${ticketId}:`, error);
    return null;
  }
};

/**
 * Create a new device
 * @param deviceData - The device data to create
 * @returns Promise<Device> - A promise that resolves to the created device
 */
export const createDevice = async (deviceData: DeviceFormData): Promise<Device> => {
  try {
    const response = await axios.post(`${API_BASE_URL}/devices`, deviceData);
    const device = response.data;
    
    // Transform the backend data to match our frontend Device interface
    return {
      id: device.id,
      name: device.name,
      hostname: device.hostname,
      serial_number: device.serial_number,
      model: device.model,
      warranty_status: device.warranty_status,
      ticket_id: device.ticket_id,
      // Add frontend-specific fields with default values
      type: deviceData.type || determineDeviceType(device.model),
      lastSeen: new Date().toISOString(),
      status: 'online',
      assignedTo: null
    };
  } catch (error) {
    console.error('Error creating device:', error);
    throw error;
  }
};

/**
 * Update a device
 * @param id - The ID of the device to update
 * @param device - The updated device data
 * @returns Promise<Device> - A promise that resolves to the updated device
 */
export const updateDevice = async (id: number, device: Partial<Device>): Promise<Device> => {
  try {
    // Convert frontend Device to backend NewDevice
    const backendDevice = {
      name: device.name,
      hostname: device.hostname,
      serial_number: device.serial_number,
      model: device.model,
      warranty_status: device.warranty_status,
      ticket_id: device.ticket_id
    };
    
    const response = await axios.put(`${API_BASE_URL}/devices/${id}`, backendDevice);
    const updatedDevice = response.data;
    
    // Transform the backend data to match our frontend Device interface
    return {
      id: updatedDevice.id,
      name: updatedDevice.name,
      hostname: updatedDevice.hostname,
      serial_number: updatedDevice.serial_number,
      model: updatedDevice.model,
      warranty_status: updatedDevice.warranty_status,
      ticket_id: updatedDevice.ticket_id,
      // Add frontend-specific fields with default values
      type: device.type || determineDeviceType(updatedDevice.model),
      lastSeen: device.lastSeen || new Date().toISOString(),
      status: device.status || 'online',
      assignedTo: device.assignedTo || null,
      specs: device.specs || getDeviceSpecs(updatedDevice.model)
    };
  } catch (error) {
    console.error(`Error updating device with ID ${id}:`, error);
    throw error;
  }
};

/**
 * Delete a device
 * @param id - The ID of the device to delete
 * @returns Promise<void>
 */
export const deleteDevice = async (id: number): Promise<void> => {
  try {
    await axios.delete(`${API_BASE_URL}/devices/${id}`);
  } catch (error) {
    console.error(`Error deleting device with ID ${id}:`, error);
    throw error;
  }
};

/**
 * Helper function to determine device type based on model
 * @param model - The device model
 * @returns string - The device type
 */
const determineDeviceType = (model: string): string => {
  const modelLower = model.toLowerCase();
  
  if (modelLower.includes('macbook') || modelLower.includes('thinkpad') || modelLower.includes('xps')) {
    return 'Laptop';
  } else if (modelLower.includes('iphone') || modelLower.includes('pixel')) {
    return 'Mobile';
  } else if (modelLower.includes('ipad') || modelLower.includes('tab')) {
    return 'Tablet';
  } else if (modelLower.includes('imac') || modelLower.includes('desktop')) {
    return 'Desktop';
  } else {
    return 'Other';
  }
};

/**
 * Helper function to get device specs based on model
 * @param model - The device model
 * @returns object - The device specs
 */
const getDeviceSpecs = (model: string): Device['specs'] => {
  const modelLower = model.toLowerCase();
  
  if (modelLower.includes('macbook') && modelLower.includes('pro')) {
    return {
      cpu: 'Apple M1 Pro',
      memory: '16GB',
      storage: '512GB SSD',
      os: 'macOS 14.0'
    };
  } else if (modelLower.includes('thinkpad')) {
    return {
      cpu: 'Intel Core i7-1165G7',
      memory: '16GB',
      storage: '512GB SSD',
      os: 'Windows 11'
    };
  } else if (modelLower.includes('xps')) {
    return {
      cpu: 'Intel Core i7-11800H',
      memory: '32GB',
      storage: '1TB SSD',
      os: 'Windows 11'
    };
  } else {
    return {
      cpu: 'Unknown',
      memory: 'Unknown',
      storage: 'Unknown',
      os: 'Unknown'
    };
  }
}; 127