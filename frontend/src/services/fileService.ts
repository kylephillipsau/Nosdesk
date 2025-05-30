// Service for handling authenticated file access
import apiConfig from './apiConfig'

// Generate an authenticated URL for a file by appending the JWT token
export const getAuthenticatedFileUrl = (filePath: string): string => {
  const token = localStorage.getItem('token')
  if (!token) {
    console.warn('No authentication token found for file access')
    return filePath // Return original path as fallback
  }

  // If the path already contains query parameters, append with &, otherwise start with ?
  const separator = filePath.includes('?') ? '&' : '?'
  return `${filePath}${separator}token=${encodeURIComponent(token)}`
}

// Convert old upload paths to new authenticated API paths
export const convertToAuthenticatedPath = (originalPath: string): string => {
  // Convert /uploads/tickets/... to /api/files/tickets/...
  if (originalPath.startsWith('/uploads/tickets/')) {
    const filename = originalPath.replace('/uploads/tickets/', '')
    return getAuthenticatedFileUrl(`/api/files/tickets/${filename}`)
  }
  
  // Convert /uploads/temp/... to /api/files/temp/...  
  if (originalPath.startsWith('/uploads/temp/')) {
    const filename = originalPath.replace('/uploads/temp/', '')
    return getAuthenticatedFileUrl(`/api/files/temp/${filename}`)
  }
  
  // For other paths (like user avatars), return as-is since they're public
  return originalPath
}

// Download a file with authentication
export const downloadAuthenticatedFile = async (filePath: string, filename?: string): Promise<void> => {
  const token = localStorage.getItem('token')
  if (!token) {
    throw new Error('No authentication token found')
  }

  try {
    const response = await fetch(getAuthenticatedFileUrl(filePath), {
      headers: {
        'Authorization': `Bearer ${token}`
      }
    })

    if (!response.ok) {
      throw new Error(`Failed to download file: ${response.statusText}`)
    }

    const blob = await response.blob()
    const url = window.URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = filename || 'download'
    document.body.appendChild(link)
    link.click()
    document.body.removeChild(link)
    window.URL.revokeObjectURL(url)
  } catch (error) {
    console.error('Error downloading file:', error)
    throw error
  }
}

export default {
  getAuthenticatedFileUrl,
  convertToAuthenticatedPath,
  downloadAuthenticatedFile
} 