use image::ImageFormat;
use tokio::fs;

/// Process and resize an uploaded avatar image to WebP format with fixed dimensions
/// This ensures consistent sizing and optimal storage
pub async fn process_avatar_image(
    image_bytes: &[u8],
    user_uuid: &str,
    max_size: u32, // Maximum width/height in pixels
) -> Result<Option<String>, String> {
    println!("Processing avatar image for user: {}, max_size: {}px", user_uuid, max_size);
    
    // Process image in a blocking task to avoid blocking the async runtime
    let user_uuid = user_uuid.to_string();
    let image_bytes = image_bytes.to_vec();
    let avatar_result = tokio::task::spawn_blocking(move || {
        // Load the image
        let img = match image::load_from_memory(&image_bytes) {
            Ok(img) => img,
            Err(e) => {
                println!("Failed to load image from memory: {}", e);
                return None;
            }
        };
        
        println!("Original image dimensions: {}x{}", img.width(), img.height());
        
        // Create a square image by center cropping to 1:1 aspect ratio
        let square_img = create_square_crop(&img, max_size);
        
        println!("Final image dimensions: {}x{}", square_img.width(), square_img.height());
        
        // Convert to WebP format
        let mut webp_bytes = Vec::new();
        match square_img.write_to(&mut std::io::Cursor::new(&mut webp_bytes), ImageFormat::WebP) {
            Ok(_) => {
                println!("Successfully converted image to WebP format");
                Some(webp_bytes)
            },
            Err(e) => {
                println!("Failed to encode image as WebP: {}", e);
                None
            }
        }
    }).await;
    
    let webp_bytes = match avatar_result {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(None),
        Err(e) => {
            println!("Avatar processing task failed: {}", e);
            return Ok(None);
        }
    };
    
    // Create avatar directory
    let avatar_dir = "uploads/users/avatars";
    if let Err(e) = fs::create_dir_all(avatar_dir).await {
        return Err(format!("Failed to create avatar directory: {}", e));
    }
    
    // Clean up any existing avatars for this user
    cleanup_old_user_avatars(avatar_dir, &user_uuid).await?;
    
    // Save the processed avatar
    let avatar_filename = format!("{}_avatar.webp", user_uuid);
    let avatar_path = format!("{}/{}", avatar_dir, avatar_filename);
    
    match fs::write(&avatar_path, &webp_bytes).await {
        Ok(_) => {
            println!("Successfully saved processed avatar to: {}", avatar_path);
            let avatar_url = format!("/{}", avatar_path);
            Ok(Some(avatar_url))
        },
        Err(e) => {
            println!("Failed to save processed avatar: {}", e);
            Ok(None)
        }
    }
}

/// Generate a 48x48 WebP thumbnail from an existing image file
pub async fn generate_user_avatar_thumbnail(
    image_path: &str,
    user_uuid: &str,
) -> Result<Option<String>, String> {
    // Convert URL path to file system path if needed
    let file_path = if image_path.starts_with('/') {
        format!(".{}", image_path) // Remove leading slash and add current directory
    } else {
        image_path.to_string()
    };
    
    println!("Generating thumbnail from: {}", file_path);
    
    // Read the original image
    let img_bytes = match fs::read(&file_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("Failed to read avatar file {}: {}", file_path, e);
            return Ok(None);
        }
    };
    
    // Process image in a blocking task to avoid blocking the async runtime
    let user_uuid = user_uuid.to_string();
    let thumbnail_result = tokio::task::spawn_blocking(move || {
        // Load the image
        let img = match image::load_from_memory(&img_bytes) {
            Ok(img) => img,
            Err(e) => {
                println!("Failed to load image from memory: {}", e);
                return None;
            }
        };
        
        // Create a square thumbnail by center cropping to 1:1 aspect ratio
        let thumbnail = create_square_crop(&img, 48);
        
        // Convert to WebP format
        let mut webp_bytes = Vec::new();
        match thumbnail.write_to(&mut std::io::Cursor::new(&mut webp_bytes), ImageFormat::WebP) {
            Ok(_) => Some(webp_bytes),
            Err(e) => {
                println!("Failed to encode image as WebP: {}", e);
                None
            }
        }
    }).await;
    
    let webp_bytes = match thumbnail_result {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(None),
        Err(e) => {
            println!("Thumbnail generation task failed: {}", e);
            return Ok(None);
        }
    };
    
    // Create thumbnail directory
    let thumb_dir = "uploads/users/thumbs";
    if let Err(e) = fs::create_dir_all(thumb_dir).await {
        return Err(format!("Failed to create thumbnail directory: {}", e));
    }
    
    // Clean up any existing thumbnails for this user
    cleanup_old_user_thumbnails(thumb_dir, &user_uuid).await?;
    
    // Save the thumbnail
    let thumb_filename = format!("{}_thumb.webp", user_uuid);
    let thumb_path = format!("{}/{}", thumb_dir, thumb_filename);
    
    match fs::write(&thumb_path, &webp_bytes).await {
        Ok(_) => {
            println!("Successfully saved thumbnail to: {}", thumb_path);
            let thumb_url = format!("/{}", thumb_path);
            Ok(Some(thumb_url))
        },
        Err(e) => {
            println!("Failed to save thumbnail: {}", e);
            Ok(None)
        }
    }
}

/// Clean up old avatar files for a specific user
async fn cleanup_old_user_avatars(
    avatar_dir: &str,
    user_uuid: &str,
) -> Result<(), String> {
    // Read the directory
    let mut dir = match fs::read_dir(avatar_dir).await {
        Ok(dir) => dir,
        Err(_) => return Ok(()), // Directory doesn't exist, nothing to clean
    };

    // Look for files matching the pattern: {user_uuid}_avatar.{ext}
    let pattern_prefix = format!("{}_avatar", user_uuid);
    
    while let Ok(Some(entry)) = dir.next_entry().await {
        if let Some(filename) = entry.file_name().to_str() {
            // Check if this file matches our pattern (user_uuid_avatar.ext)
            if filename.starts_with(&pattern_prefix) && filename.contains('.') {
                let file_path = entry.path();
                println!("Cleaning up old avatar file: {:?}", file_path);
                
                if let Err(e) = fs::remove_file(&file_path).await {
                    eprintln!("Warning: Failed to remove old avatar file {:?}: {}", file_path, e);
                    // Continue with cleanup even if one file fails
                }
            }
        }
    }
    
    Ok(())
}

/// Clean up old thumbnail files for a specific user
async fn cleanup_old_user_thumbnails(
    thumb_dir: &str,
    user_uuid: &str,
) -> Result<(), String> {
    // Read the directory
    let mut dir = match fs::read_dir(thumb_dir).await {
        Ok(dir) => dir,
        Err(_) => return Ok(()), // Directory doesn't exist, nothing to clean
    };

    // Look for files matching the pattern: {user_uuid}_thumb.{ext}
    let pattern_prefix = format!("{}_thumb", user_uuid);
    
    while let Ok(Some(entry)) = dir.next_entry().await {
        if let Some(filename) = entry.file_name().to_str() {
            // Check if this file matches our pattern (user_uuid_thumb.ext)
            if filename.starts_with(&pattern_prefix) && filename.contains('.') {
                let file_path = entry.path();
                println!("Cleaning up old thumbnail file: {:?}", file_path);
                
                if let Err(e) = fs::remove_file(&file_path).await {
                    eprintln!("Warning: Failed to remove old thumbnail file {:?}: {}", file_path, e);
                    // Continue with cleanup even if one file fails
                }
            }
        }
    }
    
    Ok(())
}

/// Process and resize an uploaded banner image to WebP format with banner dimensions
/// This ensures consistent sizing and optimal storage for banner/cover images
pub async fn process_banner_image(
    image_bytes: &[u8],
    user_uuid: &str,
    max_width: u32,  // Maximum width in pixels (e.g., 1200)
    max_height: u32, // Maximum height in pixels (e.g., 400)
) -> Result<Option<String>, String> {
    println!("Processing banner image for user: {}, max_size: {}x{}", user_uuid, max_width, max_height);
    
    // Process image in a blocking task to avoid blocking the async runtime
    let user_uuid = user_uuid.to_string();
    let image_bytes = image_bytes.to_vec();
    let banner_result = tokio::task::spawn_blocking(move || {
        // Load the image
        let img = match image::load_from_memory(&image_bytes) {
            Ok(img) => img,
            Err(e) => {
                println!("Failed to load image from memory: {}", e);
                return None;
            }
        };
        
        println!("Original banner dimensions: {}x{}", img.width(), img.height());
        
        // Create a banner-aspect image by cropping and resizing
        let banner_img = create_banner_crop(&img, max_width, max_height);
        
        println!("Final banner dimensions: {}x{}", banner_img.width(), banner_img.height());
        
        // Convert to WebP format
        let mut webp_bytes = Vec::new();
        match banner_img.write_to(&mut std::io::Cursor::new(&mut webp_bytes), ImageFormat::WebP) {
            Ok(_) => {
                println!("Successfully converted banner to WebP format");
                Some(webp_bytes)
            },
            Err(e) => {
                println!("Failed to encode banner as WebP: {}", e);
                None
            }
        }
    }).await;
    
    let webp_bytes = match banner_result {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return Ok(None),
        Err(e) => {
            println!("Banner processing task failed: {}", e);
            return Ok(None);
        }
    };
    
    // Create banner directory
    let banner_dir = "uploads/users/banners";
    if let Err(e) = fs::create_dir_all(banner_dir).await {
        return Err(format!("Failed to create banner directory: {}", e));
    }
    
    // Clean up any existing banners for this user
    cleanup_old_user_banners(banner_dir, &user_uuid).await?;
    
    // Save the processed banner
    let banner_filename = format!("{}_banner.webp", user_uuid);
    let banner_path = format!("{}/{}", banner_dir, banner_filename);
    
    match fs::write(&banner_path, &webp_bytes).await {
        Ok(_) => {
            println!("Successfully saved processed banner to: {}", banner_path);
            let banner_url = format!("/{}", banner_path);
            Ok(Some(banner_url))
        },
        Err(e) => {
            println!("Failed to save processed banner: {}", e);
            Ok(None)
        }
    }
}

/// Clean up old banner files for a specific user
async fn cleanup_old_user_banners(
    banner_dir: &str,
    user_uuid: &str,
) -> Result<(), String> {
    // Read the directory
    let mut dir = match fs::read_dir(banner_dir).await {
        Ok(dir) => dir,
        Err(_) => return Ok(()), // Directory doesn't exist, nothing to clean
    };

    // Look for files matching the pattern: {user_uuid}_banner.{ext}
    let pattern_prefix = format!("{}_banner", user_uuid);
    
    while let Ok(Some(entry)) = dir.next_entry().await {
        if let Some(filename) = entry.file_name().to_str() {
            // Check if this file matches our pattern (user_uuid_banner.ext)
            if filename.starts_with(&pattern_prefix) && filename.contains('.') {
                let file_path = entry.path();
                println!("Cleaning up old banner file: {:?}", file_path);
                
                if let Err(e) = fs::remove_file(&file_path).await {
                    eprintln!("Warning: Failed to remove old banner file {:?}: {}", file_path, e);
                    // Continue with cleanup even if one file fails
                }
            }
        }
    }
    
    Ok(())
}

/// Create a banner-aspect crop of an image optimized for banner/cover images
/// This creates a 3:1 aspect ratio by default, suitable for profile banners
fn create_banner_crop(img: &image::DynamicImage, max_width: u32, max_height: u32) -> image::DynamicImage {
    let original_width = img.width();
    let original_height = img.height();
    
    println!("Creating banner crop from {}x{} to fit within {}x{}", original_width, original_height, max_width, max_height);
    
    // Calculate the target aspect ratio
    let target_ratio = max_width as f32 / max_height as f32;
    let original_ratio = original_width as f32 / original_height as f32;
    
    let (crop_width, crop_height, crop_x, crop_y) = if original_ratio > target_ratio {
        // Image is too wide, crop horizontally
        let crop_height = original_height;
        let crop_width = (crop_height as f32 * target_ratio) as u32;
        let crop_x = (original_width - crop_width) / 2;
        let crop_y = 0;
        (crop_width, crop_height, crop_x, crop_y)
    } else {
        // Image is too tall, crop vertically
        let crop_width = original_width;
        let crop_height = (crop_width as f32 / target_ratio) as u32;
        let crop_x = 0;
        let crop_y = (original_height - crop_height) / 2;
        (crop_width, crop_height, crop_x, crop_y)
    };
    
    println!("Cropping {}x{} banner from position ({}, {})", crop_width, crop_height, crop_x, crop_y);
    
    // Create the banner crop
    let cropped_img = img.crop_imm(crop_x, crop_y, crop_width, crop_height);
    
    // Resize to fit within the maximum dimensions while maintaining aspect ratio
    let (final_width, final_height) = if crop_width > max_width || crop_height > max_height {
        let scale_w = max_width as f32 / crop_width as f32;
        let scale_h = max_height as f32 / crop_height as f32;
        let scale = scale_w.min(scale_h);
        
        let final_width = (crop_width as f32 * scale) as u32;
        let final_height = (crop_height as f32 * scale) as u32;
        
        println!("Resizing banner from {}x{} to {}x{}", crop_width, crop_height, final_width, final_height);
        (final_width, final_height)
    } else {
        (crop_width, crop_height)
    };
    
    // Resize the banner to the final dimensions
    if final_width != crop_width || final_height != crop_height {
        cropped_img.resize_exact(final_width, final_height, image::imageops::FilterType::Lanczos3)
    } else {
        cropped_img
    }
}

/// Create a square crop of an image by center cropping to achieve 1:1 aspect ratio
/// This ensures consistent square avatars regardless of the original image dimensions
fn create_square_crop(img: &image::DynamicImage, target_size: u32) -> image::DynamicImage {
    let original_width = img.width();
    let original_height = img.height();
    
    println!("Creating square crop from {}x{} to {}x{}", original_width, original_height, target_size, target_size);
    
    // Determine the size of the square crop from the original image
    let crop_size = std::cmp::min(original_width, original_height);
    
    // Calculate the top-left coordinates for center cropping
    let crop_x = (original_width - crop_size) / 2;
    let crop_y = (original_height - crop_size) / 2;
    
    println!("Cropping {}x{} square from position ({}, {})", crop_size, crop_size, crop_x, crop_y);
    
    // Create the square crop
    let cropped_img = img.crop_imm(crop_x, crop_y, crop_size, crop_size);
    
    // Resize the square crop to the target size
    let final_img = if crop_size != target_size {
        println!("Resizing cropped square from {}x{} to {}x{}", crop_size, crop_size, target_size, target_size);
        cropped_img.resize_exact(target_size, target_size, image::imageops::FilterType::Lanczos3)
    } else {
        cropped_img
    };
    
    final_img
} 