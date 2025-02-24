use crate::error::XtaskError;

/// Generate an image with a specific format.
pub fn gen_image(data: &[u8]) -> Result<Vec<u8>, XtaskError> {
    // Create a vector with 1MB of zeros as the initial offset
    let mut image = vec![0; 0x100000];

    // Append the input data to the image
    image.extend_from_slice(data);

    // Pad the image to ensure its size is a multiple of 512 bytes
    if image.len() % 512 != 0 {
        let padding_size = 512 - image.len() % 512;
        image.extend(vec![0; padding_size]);
    }

    Ok(image)
}
