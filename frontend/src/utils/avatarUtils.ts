// Avatar utility functions for handling different image sizes

export interface AvatarSizeOptions {
  preferredSize: '48x48' | '120x120' | 'default'
  fallbackSizes?: string[]
}

/**
 * Get the appropriate avatar URL based on the desired size
 * Since we have separate avatar_url and avatar_thumb fields, we can use them directly
 * @param avatarUrl - The full size avatar URL
 * @param avatarThumb - The thumbnail avatar URL
 * @param preferThumb - Whether to prefer the thumbnail version
 * @returns The appropriate URL or null if no avatar available
 */
export function getAvatarUrl(avatarUrl: string | null, avatarThumb: string | null, preferThumb: boolean = false): string | null {
  if (preferThumb && avatarThumb) {
    return avatarThumb
  }
  return avatarUrl || avatarThumb || null
}

/**
 * Get avatar URL optimized for list views (prefer thumbnail)
 */
export function getListAvatarUrl(avatarUrl: string | null, avatarThumb: string | null): string | null {
  return getAvatarUrl(avatarUrl, avatarThumb, true)
}

/**
 * Get avatar URL optimized for profile views (prefer full size)
 */
export function getProfileAvatarUrl(avatarUrl: string | null, avatarThumb: string | null): string | null {
  return getAvatarUrl(avatarUrl, avatarThumb, false)
}

/**
 * Get avatar URL for a specific size preference
 */
export function getAvatarUrlForSize(
  avatarUrl: string | null, 
  avatarThumb: string | null,
  size: '48x48' | '120x120' | 'default'
): string | null {
  // For small sizes, prefer thumbnail
  if (size === '48x48') {
    return getAvatarUrl(avatarUrl, avatarThumb, true)
  }
  // For larger sizes, prefer full avatar
  return getAvatarUrl(avatarUrl, avatarThumb, false)
}

/**
 * Preload multiple avatar sizes for better performance
 */
export async function preloadAvatarSizes(avatarUrl: string | null, avatarThumb: string | null): Promise<void> {
  const urlsToPreload = [avatarUrl, avatarThumb].filter(Boolean) as string[]
  
  if (urlsToPreload.length === 0) return

  const promises = urlsToPreload.map(url => {
    return new Promise<void>((resolve) => {
      const img = new Image()
      img.onload = () => resolve()
      img.onerror = () => resolve() // Don't fail the whole operation if one size fails
      img.src = url
    })
  })

  await Promise.allSettled(promises)
}

/**
 * Check if an avatar URL appears to have multiple sizes available
 */
export function hasMultipleSizes(avatarUrl: string | null, avatarThumb: string | null): boolean {
  return !!(avatarUrl && avatarThumb && avatarUrl !== avatarThumb)
}

/**
 * Extract the size from an avatar URL (legacy function, not needed with separate fields)
 */
export function extractSizeFromUrl(avatarUrl: string | null): string | null {
  if (!avatarUrl) return null
  
  // Simple check for common size patterns
  if (avatarUrl.includes('_thumb.')) return 'thumb'
  if (avatarUrl.includes('_120x120.')) return '120x120'
  if (avatarUrl.includes('_48x48.')) return '48x48'
  
  return 'default'
} 