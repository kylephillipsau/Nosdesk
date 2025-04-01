import axios from 'axios';

// Define the base API URL
const API_URL = import.meta.env.VITE_API_URL || '/api';

// User interface matching the backend model
export interface User {
  id: number;
  uuid: string;
  name: string;
  email: string;
  role: string;
}

// Service for user-related API calls
const userService = {
  // Get all users
  async getUsers(): Promise<User[]> {
    try {
      const response = await axios.get(`${API_URL}/users`);
      return response.data;
    } catch (error) {
      console.error('Error fetching users:', error);
      throw error;
    }
  },

  // Get a user by UUID
  async getUserByUuid(uuid: string): Promise<User> {
    try {
      const response = await axios.get(`${API_URL}/users/uuid/${uuid}`);
      return response.data;
    } catch (error) {
      console.error(`Error fetching user with UUID ${uuid}:`, error);
      throw error;
    }
  },

  // Create a new user
  async createUser(user: { name: string; email: string; role: string }): Promise<User> {
    try {
      const response = await axios.post(`${API_URL}/users`, {
        uuid: '', // The backend will generate a UUID if empty
        ...user
      });
      return response.data;
    } catch (error) {
      console.error('Error creating user:', error);
      throw error;
    }
  },

  // Update a user
  async updateUser(uuid: string, userData: Partial<User>): Promise<User> {
    try {
      const response = await axios.put(`${API_URL}/users/uuid/${uuid}`, userData);
      return response.data;
    } catch (error) {
      console.error(`Error updating user with UUID ${uuid}:`, error);
      throw error;
    }
  },

  // Delete a user
  async deleteUser(uuid: string): Promise<void> {
    try {
      await axios.delete(`${API_URL}/users/uuid/${uuid}`);
    } catch (error) {
      console.error(`Error deleting user with UUID ${uuid}:`, error);
      throw error;
    }
  }
};

export default userService; 